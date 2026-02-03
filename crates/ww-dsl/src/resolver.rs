use std::collections::HashMap;

use ww_core::entity::EntityId;

use crate::ast::{Declaration, Span};
use crate::diagnostics::Diagnostic;

/// Tracks where each source file lives within a concatenated source string.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Display name for the file (e.g., "characters.ww" or "\<source\>").
    pub name: String,
    /// Byte offset of this file's content within the concatenated source.
    pub offset: usize,
    /// Byte length of this file's content.
    pub len: usize,
}

/// Maps byte offsets in the concatenated source back to individual files.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<FileEntry>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Create a SourceMap for a single anonymous source string.
    pub fn single(source_len: usize) -> Self {
        Self {
            files: vec![FileEntry {
                name: "<source>".to_string(),
                offset: 0,
                len: source_len,
            }],
        }
    }

    /// Add a file entry. Returns the file index.
    pub fn add_file(&mut self, name: String, offset: usize, len: usize) -> usize {
        let idx = self.files.len();
        self.files.push(FileEntry { name, offset, len });
        idx
    }

    /// Find which file a byte offset belongs to.
    pub fn file_for_offset(&self, offset: usize) -> Option<&FileEntry> {
        self.files
            .iter()
            .find(|f| offset >= f.offset && offset < f.offset + f.len)
    }

    /// Find file index for a byte offset.
    pub fn file_index_for_offset(&self, offset: usize) -> Option<usize> {
        self.files
            .iter()
            .position(|f| offset >= f.offset && offset < f.offset + f.len)
    }

    /// Get a file entry by index.
    pub fn get_file(&self, index: usize) -> Option<&FileEntry> {
        self.files.get(index)
    }

    /// Get all file entries.
    pub fn files(&self) -> &[FileEntry] {
        &self.files
    }

    /// Convert a global span to a local span within its file.
    /// Returns `(file_index, local_span)`.
    pub fn to_local_span(&self, span: &Span) -> Option<(usize, Span)> {
        self.file_index_for_offset(span.start).map(|idx| {
            let file = &self.files[idx];
            let local_start = span.start - file.offset;
            let local_end = span.end - file.offset;
            (idx, local_start..local_end)
        })
    }
}

/// Information about a resolved entity name.
#[derive(Debug, Clone)]
pub struct ResolvedEntity {
    /// Pre-assigned entity ID.
    pub id: EntityId,
    /// Index into the SourceMap for the file where this entity is declared.
    pub file_index: usize,
    /// Span of the entity name in the concatenated source.
    pub name_span: Span,
}

/// Cross-file name resolver for the Weltenwanderer DSL.
///
/// The Resolver pre-scans all entity declarations, assigns EntityIds,
/// and detects duplicates. The compiler then uses [`Resolver::lookup`] for
/// name resolution instead of maintaining its own map.
pub struct Resolver {
    /// Map from lowercased entity name to resolution info.
    names: HashMap<String, ResolvedEntity>,
    /// Diagnostics produced during resolution (e.g., duplicate names).
    pub diagnostics: Vec<Diagnostic>,
}

impl Resolver {
    /// Build a Resolver by scanning all declarations in the AST.
    pub fn resolve(ast: &crate::ast::SourceFile, source_map: &SourceMap) -> Self {
        let mut names: HashMap<String, ResolvedEntity> = HashMap::new();
        let mut diagnostics = Vec::new();

        for decl in &ast.declarations {
            if let Declaration::Entity(entity_decl) = &decl.node {
                let name_lower = entity_decl.name.node.to_lowercase();
                let name_span = entity_decl.name.span.clone();

                let file_index = source_map
                    .file_index_for_offset(name_span.start)
                    .unwrap_or(0);

                if let Some(existing) = names.get(&name_lower) {
                    // Duplicate entity — produce file-aware diagnostic
                    let existing_file = source_map
                        .get_file(existing.file_index)
                        .map(|f| f.name.as_str())
                        .unwrap_or("<unknown>");
                    let current_file = source_map
                        .get_file(file_index)
                        .map(|f| f.name.as_str())
                        .unwrap_or("<unknown>");

                    let message = if existing_file == current_file {
                        format!("entity already exists: \"{}\"", entity_decl.name.node)
                    } else {
                        format!(
                            "entity already exists: \"{}\" (also defined in {})",
                            entity_decl.name.node, existing_file
                        )
                    };

                    diagnostics.push(
                        Diagnostic::error(name_span, message)
                            .with_label(format!("first defined in {existing_file}")),
                    );
                } else {
                    let id = EntityId::new();
                    names.insert(
                        name_lower,
                        ResolvedEntity {
                            id,
                            file_index,
                            name_span,
                        },
                    );
                }
            }
        }

        Self { names, diagnostics }
    }

    /// Look up an entity by name. Returns the pre-assigned EntityId.
    /// On failure, pushes an "undefined entity" diagnostic with file context.
    pub fn lookup(
        &self,
        name: &str,
        span: &Span,
        source_map: &SourceMap,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<EntityId> {
        let lower = name.to_lowercase();
        if let Some(resolved) = self.names.get(&lower) {
            Some(resolved.id)
        } else {
            let file_hint = source_map
                .file_for_offset(span.start)
                .map(|f| format!("not defined in {}", f.name))
                .unwrap_or_else(|| "not defined in any .ww file".to_string());

            diagnostics.push(
                Diagnostic::error(span.clone(), format!("undefined entity: \"{name}\""))
                    .with_label(file_hint),
            );
            None
        }
    }

    /// Get the resolved entity info for a name (no diagnostic on miss).
    pub fn get(&self, name: &str) -> Option<&ResolvedEntity> {
        self.names.get(&name.to_lowercase())
    }

    /// Number of resolved entities.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether no entities were resolved.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    fn parse_source(source: &str) -> crate::ast::SourceFile {
        let (tokens, lex_errors) = lexer::lex(source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        parser::parse(&tokens).expect("parse error")
    }

    #[test]
    fn single_entity_resolution() {
        let source = "Kael is a character { species human }";
        let ast = parse_source(source);
        let sm = SourceMap::single(source.len());
        let resolver = Resolver::resolve(&ast, &sm);

        assert_eq!(resolver.len(), 1);
        assert!(resolver.diagnostics.is_empty());

        let resolved = resolver.get("Kael").unwrap();
        assert_eq!(resolved.file_index, 0);
    }

    #[test]
    fn case_insensitive_lookup() {
        let source = "Kael Stormborn is a character { species human }";
        let ast = parse_source(source);
        let sm = SourceMap::single(source.len());
        let resolver = Resolver::resolve(&ast, &sm);

        assert!(resolver.get("kael stormborn").is_some());
        assert!(resolver.get("KAEL STORMBORN").is_some());
        assert!(resolver.get("nobody").is_none());
    }

    #[test]
    fn duplicate_detection() {
        let source = "Kael is a character { species human }\nKael is a faction { type guild }";
        let ast = parse_source(source);
        let sm = SourceMap::single(source.len());
        let resolver = Resolver::resolve(&ast, &sm);

        // Only the first definition is kept
        assert_eq!(resolver.len(), 1);
        // Duplicate produces a diagnostic
        assert_eq!(resolver.diagnostics.len(), 1);
        assert!(
            resolver.diagnostics[0]
                .message
                .contains("entity already exists")
        );
    }

    #[test]
    fn cross_file_duplicate_diagnostic() {
        // Simulate two files concatenated
        let file_a = "Kael is a character { species human }";
        let file_b = "Kael is a faction { type guild }";
        let source = format!("{file_a}\n{file_b}");

        let ast = parse_source(&source);
        let mut sm = SourceMap::new();
        sm.add_file("characters.ww".into(), 0, file_a.len());
        sm.add_file("factions.ww".into(), file_a.len() + 1, file_b.len());
        let resolver = Resolver::resolve(&ast, &sm);

        assert_eq!(resolver.diagnostics.len(), 1);
        assert!(
            resolver.diagnostics[0]
                .message
                .contains("also defined in characters.ww"),
            "diagnostic should mention the other file: {}",
            resolver.diagnostics[0].message
        );
    }

    #[test]
    fn undefined_entity_diagnostic() {
        let source = "Kael is a character { species human }";
        let ast = parse_source(source);
        let sm = SourceMap::single(source.len());
        let resolver = Resolver::resolve(&ast, &sm);

        let mut diags = Vec::new();
        let result = resolver.lookup("Nobody", &(0..6), &sm, &mut diags);

        assert!(result.is_none());
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("undefined entity"));
    }

    #[test]
    fn source_map_single() {
        let sm = SourceMap::single(100);

        assert!(sm.file_for_offset(0).is_some());
        assert!(sm.file_for_offset(99).is_some());
        assert!(sm.file_for_offset(100).is_none());

        assert_eq!(sm.file_for_offset(50).unwrap().name, "<source>");
    }

    #[test]
    fn source_map_multi_file() {
        let mut sm = SourceMap::new();
        sm.add_file("a.ww".into(), 0, 50);
        sm.add_file("b.ww".into(), 51, 30); // 1-byte gap for newline separator

        assert_eq!(sm.file_for_offset(0).unwrap().name, "a.ww");
        assert_eq!(sm.file_for_offset(49).unwrap().name, "a.ww");
        assert!(sm.file_for_offset(50).is_none()); // newline separator gap
        assert_eq!(sm.file_for_offset(51).unwrap().name, "b.ww");
        assert_eq!(sm.file_for_offset(80).unwrap().name, "b.ww");
        assert!(sm.file_for_offset(81).is_none());
    }

    #[test]
    fn source_map_to_local_span() {
        let mut sm = SourceMap::new();
        sm.add_file("a.ww".into(), 0, 50);
        sm.add_file("b.ww".into(), 51, 30);

        // Span in first file
        let (idx, local) = sm.to_local_span(&(10..20)).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(local, 10..20);

        // Span in second file
        let (idx, local) = sm.to_local_span(&(55..65)).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(local, 4..14);
    }
}
