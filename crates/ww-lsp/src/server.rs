use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use ww_dsl::ast::{Declaration, SourceFile, Statement};
use ww_dsl::diagnostics::Severity;
use ww_dsl::lexer::Token;
use ww_dsl::resolver::{Resolver, SourceMap};
use ww_dsl::{compiler, lexer, parser};

/// Tracks where each file's content sits within the concatenated source.
struct FileSlice {
    uri: Url,
    offset: usize,
    len: usize,
    text: String,
}

/// Info about a compiled entity, now with its defining file URI.
struct EntityInfo {
    name: String,
    kind: String,
    /// Byte span within that file's own text (not the concatenated source).
    local_span: std::ops::Range<usize>,
    /// Which file this entity is defined in.
    uri: Url,
}

/// Shared workspace state.
struct WorkspaceState {
    /// Open document texts (in-memory, may be unsaved).
    open_docs: HashMap<Url, String>,
    /// All entities from the last successful workspace compilation.
    entities: Vec<EntityInfo>,
    /// All entity names (for autocomplete).
    entity_names: Vec<String>,
    /// Workspace root path.
    root: Option<PathBuf>,
    /// Parsed AST from the last compilation.
    ast: Option<SourceFile>,
    /// Lexed tokens with spans from the last compilation.
    tokens: Option<Vec<(Token, std::ops::Range<usize>)>>,
    /// File slices for mapping global offsets to per-file locations.
    slices: Vec<FileSlice>,
    /// Concatenated source text from all .ww files.
    concatenated_source: String,
    /// Source map for the ww-dsl compiler.
    source_map: Option<SourceMap>,
    /// Hash of the last compiled source (for incremental compilation).
    last_source_hash: u64,
}

pub struct WwLanguageServer {
    client: Client,
    state: Arc<RwLock<WorkspaceState>>,
}

impl WwLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(WorkspaceState {
                open_docs: HashMap::new(),
                entities: Vec::new(),
                entity_names: Vec::new(),
                root: None,
                ast: None,
                tokens: None,
                slices: Vec::new(),
                concatenated_source: String::new(),
                source_map: None,
                last_source_hash: 0,
            })),
        }
    }

    /// Recompile the entire workspace and publish diagnostics for all files.
    async fn analyze_workspace(&self) {
        let state = self.state.read().await;
        let root = match &state.root {
            Some(r) => r.clone(),
            None => return,
        };
        let open_docs = state.open_docs.clone();
        let last_hash = state.last_source_hash;
        drop(state);

        // Discover all .ww files recursively
        let mut file_paths: Vec<PathBuf> = Vec::new();
        collect_ww_files(&root, &mut file_paths);
        file_paths.sort();

        // Build file slices + SourceMap
        let mut slices: Vec<FileSlice> = Vec::new();
        let mut concatenated = String::new();
        let mut dsl_source_map = SourceMap::new();

        for path in &file_paths {
            let uri = match Url::from_file_path(path) {
                Ok(u) => u,
                Err(()) => match Url::parse(&format!("file://{}", path.display())) {
                    Ok(u) => u,
                    Err(_) => continue,
                },
            };

            let text = if let Some(open_text) = open_docs.get(&uri) {
                open_text.clone()
            } else {
                match std::fs::read_to_string(path) {
                    Ok(t) => t,
                    Err(_) => continue,
                }
            };

            let len = text.len();

            if !concatenated.is_empty() {
                concatenated.push('\n');
            }
            let offset = concatenated.len();
            concatenated.push_str(&text);

            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            dsl_source_map.add_file(file_name, offset, len);

            slices.push(FileSlice {
                uri,
                offset,
                len,
                text,
            });
        }

        if concatenated.is_empty() {
            return;
        }

        // Incremental compilation: skip if source unchanged
        let mut hasher = DefaultHasher::new();
        concatenated.hash(&mut hasher);
        let source_hash = hasher.finish();
        if source_hash == last_hash && last_hash != 0 {
            return;
        }

        // Step-by-step pipeline (matching compile_with_source_map in ww-dsl)
        let (tokens, lex_errors) = lexer::lex(&concatenated);

        let mut diagnostics: Vec<ww_dsl::Diagnostic> = lex_errors
            .into_iter()
            .map(|e| ww_dsl::Diagnostic::error(e.span, e.message))
            .collect();

        let (ast, parse_errors) = parser::parse_lenient(&tokens);
        diagnostics.extend(
            parse_errors
                .into_iter()
                .map(|e| ww_dsl::Diagnostic::error(e.span, e.message)),
        );

        let resolver = Resolver::resolve(&ast, &dsl_source_map);
        let mut result = compiler::compile(&ast, &resolver, dsl_source_map);

        diagnostics.append(&mut result.diagnostics);
        result.diagnostics = diagnostics;

        let per_file_diags = map_diagnostics_to_files(&slices, &result.diagnostics);

        // Build entity info
        let mut entities = Vec::new();
        let mut entity_names = Vec::new();

        for entity in result.world.all_entities() {
            entity_names.push(entity.name.clone());

            let def_pos = find_definition_offset(&concatenated, &entity.name);
            if let Some(global_start) = def_pos
                && let Some(slice) = find_slice_for_offset(&slices, global_start)
            {
                let local_start = global_start - slice.offset;
                let local_end = local_start + entity.name.len();
                entities.push(EntityInfo {
                    name: entity.name.clone(),
                    kind: entity.kind.to_string(),
                    local_span: local_start..local_end,
                    uri: slice.uri.clone(),
                });
            }
        }

        // Update state with all compilation artifacts
        {
            let mut state = self.state.write().await;
            state.entities = entities;
            state.entity_names = entity_names;
            state.ast = Some(ast);
            state.tokens = Some(tokens);
            state.slices = slices;
            state.concatenated_source = concatenated;
            state.source_map = Some(result.source_map);
            state.last_source_hash = source_hash;
        }

        // Publish diagnostics for each file
        for (uri, diags) in per_file_diags {
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }
}

/// Recursively collect all .ww files under a directory.
fn collect_ww_files(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-source dirs
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
            {
                collect_ww_files(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "ww") {
            out.push(path);
        }
    }
}

/// Find which file slice a byte span falls into.
fn find_slice_for_span<'a>(
    slices: &'a [FileSlice],
    span: &std::ops::Range<usize>,
) -> Option<&'a FileSlice> {
    find_slice_for_offset(slices, span.start)
}

/// Find the byte offset of an entity's definition (the "Name is a ..." declaration).
/// Falls back to the first occurrence if no definition pattern is found.
fn find_definition_offset(text: &str, name: &str) -> Option<usize> {
    let text_lower = text.to_lowercase();
    let name_lower = name.to_lowercase();

    // Search for "<name> is a " or "<name> is an " which marks the definition
    let mut search_from = 0;
    while let Some(found) = text_lower[search_from..].find(&name_lower) {
        let start = search_from + found;
        let after_name = start + name.len();

        // Check what follows the name
        let rest = &text_lower[after_name..];
        if rest.starts_with(" is a ") || rest.starts_with(" is an ") {
            return Some(start);
        }
        search_from = start + 1;
    }

    // Fallback: first occurrence
    text_lower.find(&name_lower)
}

/// Find which file slice a byte offset falls into.
fn find_slice_for_offset(slices: &[FileSlice], offset: usize) -> Option<&FileSlice> {
    slices
        .iter()
        .find(|s| offset >= s.offset && offset < s.offset + s.len)
}

fn byte_span_to_range(text: &str, span: &std::ops::Range<usize>) -> Range {
    let start = byte_offset_to_position(text, span.start);
    let end = byte_offset_to_position(text, span.end);
    Range { start, end }
}

fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let offset = offset.min(text.len());
    let prefix = &text[..offset];
    let line = prefix.matches('\n').count() as u32;
    let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = (offset - last_newline) as u32;
    Position { line, character }
}

fn position_to_byte_offset(text: &str, pos: Position) -> usize {
    let mut offset = 0;
    for (i, line) in text.lines().enumerate() {
        if i == pos.line as usize {
            return offset + (pos.character as usize).min(line.len());
        }
        offset += line.len() + 1;
    }
    text.len()
}

/// Map DSL diagnostics to per-file LSP diagnostics.
fn map_diagnostics_to_files(
    slices: &[FileSlice],
    diagnostics: &[ww_dsl::Diagnostic],
) -> HashMap<Url, Vec<Diagnostic>> {
    let mut per_file: HashMap<Url, Vec<Diagnostic>> = HashMap::new();
    for slice in slices {
        per_file.entry(slice.uri.clone()).or_default();
    }
    for diag in diagnostics {
        if let Some(slice) = find_slice_for_span(slices, &diag.span) {
            let local_start = diag.span.start.saturating_sub(slice.offset);
            let local_end = diag.span.end.saturating_sub(slice.offset).min(slice.len);
            let local_span = local_start..local_end;
            let range = byte_span_to_range(&slice.text, &local_span);
            let severity = match diag.severity {
                Severity::Error => Some(DiagnosticSeverity::ERROR),
                Severity::Warning => Some(DiagnosticSeverity::WARNING),
            };
            per_file
                .entry(slice.uri.clone())
                .or_default()
                .push(Diagnostic {
                    range,
                    severity,
                    source: Some("ww".into()),
                    message: diag.message.clone(),
                    ..Default::default()
                });
        }
    }
    per_file
}

/// Map an entity kind string to an LSP SymbolKind.
fn entity_kind_to_symbol_kind(kind: &str) -> SymbolKind {
    match kind.to_lowercase().as_str() {
        "character" => SymbolKind::CLASS,
        "location" | "fortress" | "city" | "town" | "village" | "region" | "continent" | "room"
        | "wilderness" | "dungeon" | "building" | "landmark" | "plane" => SymbolKind::NAMESPACE,
        "faction" => SymbolKind::STRUCT,
        "event" => SymbolKind::EVENT,
        "item" => SymbolKind::OBJECT,
        "lore" => SymbolKind::FILE,
        _ => SymbolKind::VARIABLE,
    }
}

/// Find the entity name at the cursor position, preferring longer names.
fn entity_at_cursor(state: &WorkspaceState, uri: &Url, pos: Position) -> Option<String> {
    let text = get_file_text(state, uri);
    let offset = position_to_byte_offset(&text, pos);
    let text_lower = text.to_lowercase();

    let mut names: Vec<&String> = state.entity_names.iter().collect();
    names.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for name in names {
        let name_lower = name.to_lowercase();
        let mut search_from = 0;
        while let Some(found) = text_lower[search_from..].find(&name_lower) {
            let start = search_from + found;
            let end = start + name.len();
            if offset >= start && offset <= end {
                return Some(name.clone());
            }
            search_from = start + 1;
        }
    }
    None
}

/// Find all references to an entity in the AST (definitions + references).
/// Returns (global_byte_span, is_definition) pairs.
fn find_all_entity_references(ast: &SourceFile, name: &str) -> Vec<(std::ops::Range<usize>, bool)> {
    let name_lower = name.to_lowercase();
    let mut refs = Vec::new();

    for decl in &ast.declarations {
        match &decl.node {
            Declaration::Entity(entity) => {
                if entity.name.node.to_lowercase() == name_lower {
                    refs.push((entity.name.span.clone(), true));
                }
                // Walk inline annotations for target references
                for ann in &entity.annotations {
                    for target in &ann.node.targets {
                        if target.node.to_lowercase() == name_lower {
                            refs.push((target.span.clone(), false));
                        }
                    }
                }
                collect_refs_in_body(&entity.body, &name_lower, &mut refs);
            }
            Declaration::World(world) => {
                collect_refs_in_body(&world.body, &name_lower, &mut refs);
            }
        }
    }

    refs
}

/// Recursively collect entity name references from a statement body (handles nested blocks).
fn collect_refs_in_body(
    body: &[ww_dsl::ast::Spanned<Statement>],
    name_lower: &str,
    refs: &mut Vec<(std::ops::Range<usize>, bool)>,
) {
    for stmt in body {
        match &stmt.node {
            Statement::Relationship(rel) => {
                for target in &rel.targets {
                    if target.node.to_lowercase() == name_lower {
                        refs.push((target.span.clone(), false));
                    }
                }
            }
            Statement::Exit(exit) => {
                if exit.target.node.to_lowercase() == name_lower {
                    refs.push((exit.target.span.clone(), false));
                }
            }
            Statement::Block(block) => {
                collect_refs_in_body(&block.body, name_lower, refs);
            }
            _ => {}
        }
    }
}

/// Convert a global byte span to an LSP Location using file slices.
fn global_span_to_location(
    slices: &[FileSlice],
    span: &std::ops::Range<usize>,
) -> Option<Location> {
    let slice = find_slice_for_span(slices, span)?;
    let local_start = span.start.saturating_sub(slice.offset);
    let local_end = span.end.saturating_sub(slice.offset).min(slice.len);
    let range = byte_span_to_range(&slice.text, &(local_start..local_end));
    Some(Location {
        uri: slice.uri.clone(),
        range,
    })
}

/// Get the text of the current line up to the cursor position.
fn get_line_prefix(text: &str, pos: Position) -> String {
    for (i, line) in text.lines().enumerate() {
        if i == pos.line as usize {
            let char_idx = (pos.character as usize).min(line.len());
            return line[..char_idx].to_string();
        }
    }
    String::new()
}

/// Completion context detected from the line prefix.
#[derive(Debug)]
enum WwCompletionCtx {
    /// After "is a" / "is an" — suggest entity kinds.
    EntityKind,
    /// After a relationship keyword — suggest entity names.
    RelationshipTarget,
    /// Inside entity body on a new/partial line — suggest properties + relationships.
    EntityBody,
    /// After a known property key — suggest values.
    PropertyValue(String),
    /// Default context — suggest everything.
    Default,
}

/// Detect the completion context from the text before the cursor.
fn detect_completion_context(line_prefix: &str) -> WwCompletionCtx {
    let lower = line_prefix.to_lowercase();
    let trimmed_lower = lower.trim_end();

    if trimmed_lower.ends_with("is a") || trimmed_lower.ends_with("is an") {
        return WwCompletionCtx::EntityKind;
    }

    let rel_keywords = [
        "member of",
        "located at",
        "allied with",
        "rival of",
        "owned by",
        "led by",
        "based at",
        "involving",
        "references",
        "caused by",
        "north to",
        "south to",
        "east to",
        "west to",
        "up to",
        "down to",
    ];
    for kw in &rel_keywords {
        if trimmed_lower.ends_with(kw) {
            return WwCompletionCtx::RelationshipTarget;
        }
    }

    let property_keys = [
        "species",
        "occupation",
        "status",
        "climate",
        "population",
        "terrain",
        "type",
        "rarity",
        "source",
        "alignment",
        "traits",
        "values",
    ];
    for key in &property_keys {
        if trimmed_lower.ends_with(key) {
            return WwCompletionCtx::PropertyValue(key.to_string());
        }
    }

    let trimmed = line_prefix.trim();
    if line_prefix.starts_with(|c: char| c.is_whitespace())
        && (trimmed.is_empty() || !trimmed.contains(' '))
    {
        return WwCompletionCtx::EntityBody;
    }

    WwCompletionCtx::Default
}

/// Classify a token for semantic highlighting. Returns None to skip.
fn classify_token(token: &Token, next: Option<&Token>) -> Option<u32> {
    match token {
        Token::Word(w) if is_semantic_keyword(w) => Some(0), // KEYWORD
        Token::Word(_) if next.is_some_and(is_value_start) => Some(2), // PROPERTY
        Token::Str(_) => Some(3),                            // STRING
        Token::Integer(_) | Token::Float(_) => Some(4),      // NUMBER
        Token::DocString(_) => Some(5),                      // COMMENT
        Token::LBrace
        | Token::RBrace
        | Token::LBracket
        | Token::RBracket
        | Token::LParen
        | Token::RParen
        | Token::Comma => Some(6), // OPERATOR
        _ => None,
    }
}

/// Check if a word is a DSL keyword for semantic highlighting.
fn is_semantic_keyword(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "world"
            | "is"
            | "a"
            | "an"
            | "member"
            | "of"
            | "located"
            | "at"
            | "allied"
            | "with"
            | "rival"
            | "owned"
            | "by"
            | "led"
            | "based"
            | "involving"
            | "references"
            | "caused"
            | "in"
            | "leader"
            | "owner"
            | "north"
            | "south"
            | "east"
            | "west"
            | "up"
            | "down"
            | "to"
            | "date"
            | "year"
            | "month"
            | "day"
            | "era"
    )
}

/// Check if a token can start a property value.
fn is_value_start(token: &Token) -> bool {
    matches!(
        token,
        Token::Str(_) | Token::Integer(_) | Token::Float(_) | Token::LBracket | Token::Word(_)
    )
}

/// Collect all byte spans where entity names appear in the text.
fn collect_entity_name_spans(text: &str, entity_names: &[String]) -> Vec<std::ops::Range<usize>> {
    let text_lower = text.to_lowercase();
    let mut spans = Vec::new();
    for name in entity_names {
        let name_lower = name.to_lowercase();
        let mut search_from = 0;
        while let Some(found) = text_lower[search_from..].find(&name_lower) {
            let start = search_from + found;
            let end = start + name.len();
            spans.push(start..end);
            search_from = start + 1;
        }
    }
    spans
}

/// Check if a token span falls within any entity name span.
fn is_in_entity_span(
    entity_spans: &[std::ops::Range<usize>],
    token_span: &std::ops::Range<usize>,
) -> bool {
    entity_spans
        .iter()
        .any(|s| token_span.start >= s.start && token_span.end <= s.end)
}

/// Extract an entity name from an "undefined entity" diagnostic message.
fn extract_entity_name_from_diagnostic(msg: &str) -> Option<&str> {
    let prefix = "undefined entity: \"";
    let start = msg.find(prefix)? + prefix.len();
    let end = start + msg[start..].find('"')?;
    Some(&msg[start..end])
}

static KEYWORDS: &[&str] = &[
    "world",
    "is",
    "a",
    "an",
    "in",
    "member of",
    "located at",
    "allied with",
    "rival of",
    "owned by",
    "led by",
    "based at",
    "involving",
    "references",
    "caused by",
    "date",
    "year",
    "month",
    "day",
    "north to",
    "south to",
    "east to",
    "west to",
    "up to",
    "down to",
    "location",
    "character",
    "faction",
    "event",
    "item",
    "lore",
    "fortress",
    "city",
    "region",
    "room",
    "species",
    "occupation",
    "status",
    "traits",
    "climate",
    "population",
    "terrain",
    "type",
    "rarity",
    "source",
    "values",
    "alignment",
];

#[tower_lsp::async_trait]
impl LanguageServer for WwLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Extract workspace root
        let root = params
            .workspace_folders
            .as_ref()
            .and_then(|folders| folders.first())
            .and_then(|f| f.uri.to_file_path().ok())
            .or_else(|| params.root_uri.as_ref().and_then(|u| u.to_file_path().ok()));

        if let Some(root) = root {
            let mut state = self.state.write().await;
            state.root = Some(root);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![" ".into()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        },
                    ),
                ),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Weltenwanderer LSP initialized")
            .await;

        // Initial workspace analysis
        self.analyze_workspace().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store open doc text, then recompile workspace
        {
            let mut state = self.state.write().await;

            // If no workspace root set yet, derive it from the file's parent directory
            if state.root.is_none()
                && let Ok(path) = uri.to_file_path()
                && let Some(parent) = path.parent()
            {
                state.root = Some(parent.to_path_buf());
            }

            state.open_docs.insert(uri, text);
        }

        self.analyze_workspace().await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            {
                let mut state = self.state.write().await;
                state.open_docs.insert(uri, change.text);
            }
            self.analyze_workspace().await;
        }
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // Recompile on save (disk state may differ from open docs)
        self.analyze_workspace().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        {
            let mut state = self.state.write().await;
            state.open_docs.remove(&params.text_document.uri);
        }
        self.analyze_workspace().await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let state = self.state.read().await;

        let text = get_file_text(&state, &uri);
        let line_prefix = get_line_prefix(&text, pos);
        let context = detect_completion_context(&line_prefix);

        let mut items = Vec::new();

        match context {
            WwCompletionCtx::EntityKind => {
                let kinds = [
                    "character",
                    "location",
                    "fortress",
                    "city",
                    "town",
                    "village",
                    "region",
                    "continent",
                    "room",
                    "wilderness",
                    "dungeon",
                    "building",
                    "landmark",
                    "plane",
                    "faction",
                    "event",
                    "item",
                    "lore",
                ];
                for (i, kind) in kinds.iter().enumerate() {
                    items.push(CompletionItem {
                        label: kind.to_string(),
                        kind: Some(CompletionItemKind::TYPE_PARAMETER),
                        sort_text: Some(format!("0{:04}", i)),
                        detail: Some("entity kind".into()),
                        ..Default::default()
                    });
                }
            }
            WwCompletionCtx::RelationshipTarget => {
                for (i, name) in state.entity_names.iter().enumerate() {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::REFERENCE),
                        sort_text: Some(format!("0{:04}", i)),
                        detail: Some("entity".into()),
                        ..Default::default()
                    });
                }
            }
            WwCompletionCtx::EntityBody => {
                let property_keys = [
                    "species",
                    "occupation",
                    "status",
                    "climate",
                    "population",
                    "terrain",
                    "type",
                    "rarity",
                    "source",
                    "alignment",
                    "traits",
                    "values",
                ];
                for (i, key) in property_keys.iter().enumerate() {
                    items.push(CompletionItem {
                        label: key.to_string(),
                        kind: Some(CompletionItemKind::PROPERTY),
                        sort_text: Some(format!("0{:04}", i)),
                        detail: Some("property".into()),
                        ..Default::default()
                    });
                }
                let rel_keywords = [
                    "member of",
                    "located at",
                    "allied with",
                    "rival of",
                    "owned by",
                    "led by",
                    "based at",
                    "in",
                    "involving",
                    "references",
                    "caused by",
                    "north to",
                    "south to",
                    "east to",
                    "west to",
                    "up to",
                    "down to",
                ];
                for (i, kw) in rel_keywords.iter().enumerate() {
                    items.push(CompletionItem {
                        label: kw.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        sort_text: Some(format!("1{:04}", i)),
                        detail: Some("relationship".into()),
                        ..Default::default()
                    });
                }
                items.push(CompletionItem {
                    label: "date".to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    sort_text: Some("19999".into()),
                    detail: Some("date literal".into()),
                    ..Default::default()
                });
            }
            WwCompletionCtx::PropertyValue(ref key) => {
                let values: &[&str] = match key.as_str() {
                    "species" => &["human", "elf", "dwarf", "halfling", "orc", "goblin"],
                    "status" => &["alive", "dead", "missing", "unknown"],
                    "alignment" => &["good", "evil", "neutral", "lawful", "chaotic"],
                    "climate" => &["arid", "temperate", "tropical", "arctic", "desert"],
                    "terrain" => &[
                        "mountain",
                        "forest",
                        "plains",
                        "swamp",
                        "coastal",
                        "underground",
                    ],
                    "rarity" => &["common", "uncommon", "rare", "legendary", "unique"],
                    _ => &[],
                };
                for (i, val) in values.iter().enumerate() {
                    items.push(CompletionItem {
                        label: val.to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        sort_text: Some(format!("0{:04}", i)),
                        detail: Some(format!("{key} value")),
                        ..Default::default()
                    });
                }
            }
            WwCompletionCtx::Default => {
                for (i, name) in state.entity_names.iter().enumerate() {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::REFERENCE),
                        sort_text: Some(format!("0{:04}", i)),
                        detail: Some("entity".into()),
                        ..Default::default()
                    });
                }
                for (i, kw) in KEYWORDS.iter().enumerate() {
                    items.push(CompletionItem {
                        label: (*kw).to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        sort_text: Some(format!("1{:04}", i)),
                        ..Default::default()
                    });
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let state = self.state.read().await;

        // Get the text of the current file
        let text = match state.open_docs.get(&uri) {
            Some(t) => t.clone(),
            None => {
                if let Ok(path) = uri.to_file_path() {
                    std::fs::read_to_string(path).unwrap_or_default()
                } else {
                    return Ok(None);
                }
            }
        };

        let offset = position_to_byte_offset(&text, pos);
        let word_at_cursor = find_word_at_offset(&text, offset);

        if word_at_cursor.is_empty() {
            return Ok(None);
        }

        // Search for entity name match (supports cross-file go-to-def)
        for entity in &state.entities {
            let name_lower = entity.name.to_lowercase();
            let text_lower = text.to_lowercase();

            // Check if cursor is inside a reference to this entity
            let mut search_from = 0;
            while let Some(found) = text_lower[search_from..].find(&name_lower) {
                let start = search_from + found;
                let end = start + entity.name.len();
                if offset >= start && offset <= end {
                    let def_range =
                        byte_span_to_range(&get_file_text(&state, &entity.uri), &entity.local_span);
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: entity.uri.clone(),
                        range: def_range,
                    })));
                }
                search_from = start + 1;
            }
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let state = self.state.read().await;

        let text = match state.open_docs.get(&uri) {
            Some(t) => t.clone(),
            None => {
                if let Ok(path) = uri.to_file_path() {
                    std::fs::read_to_string(path).unwrap_or_default()
                } else {
                    return Ok(None);
                }
            }
        };

        let offset = position_to_byte_offset(&text, pos);

        for entity in &state.entities {
            let text_lower = text.to_lowercase();
            let name_lower = entity.name.to_lowercase();
            let mut search_from = 0;
            while let Some(found) = text_lower[search_from..].find(&name_lower) {
                let start = search_from + found;
                let end = start + entity.name.len();
                if offset >= start && offset <= end {
                    let defined_in = entity
                        .uri
                        .to_file_path()
                        .ok()
                        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                        .unwrap_or_default();

                    let hover_text = format!(
                        "**{}** [{}]\n\nDefined in `{}`",
                        entity.name, entity.kind, defined_in
                    );
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: Some(byte_span_to_range(&text, &(start..end))),
                    }));
                }
                search_from = start + 1;
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let state = self.state.read().await;

        let ast = match &state.ast {
            Some(a) => a,
            None => return Ok(None),
        };

        let file_slice = match state.slices.iter().find(|s| s.uri == uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        let mut symbols = Vec::new();

        for decl in &ast.declarations {
            match &decl.node {
                Declaration::Entity(entity) => {
                    let name_start = entity.name.span.start;
                    if name_start >= file_slice.offset
                        && name_start < file_slice.offset + file_slice.len
                    {
                        let local_name_start = name_start - file_slice.offset;
                        let local_name_end = entity.name.span.end - file_slice.offset;
                        let local_decl_start = decl.span.start.saturating_sub(file_slice.offset);
                        let local_decl_end = decl
                            .span
                            .end
                            .saturating_sub(file_slice.offset)
                            .min(file_slice.len);

                        let selection_range = byte_span_to_range(
                            &file_slice.text,
                            &(local_name_start..local_name_end),
                        );
                        let range = byte_span_to_range(
                            &file_slice.text,
                            &(local_decl_start..local_decl_end),
                        );

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: entity.name.node.clone(),
                            detail: Some(entity.kind.node.clone()),
                            kind: entity_kind_to_symbol_kind(&entity.kind.node),
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range,
                            children: None,
                        });
                    }
                }
                Declaration::World(world) => {
                    let name_start = world.name.span.start;
                    if name_start >= file_slice.offset
                        && name_start < file_slice.offset + file_slice.len
                    {
                        let local_name_start = name_start - file_slice.offset;
                        let local_name_end = world.name.span.end - file_slice.offset;
                        let local_decl_start = decl.span.start.saturating_sub(file_slice.offset);
                        let local_decl_end = decl
                            .span
                            .end
                            .saturating_sub(file_slice.offset)
                            .min(file_slice.len);

                        let selection_range = byte_span_to_range(
                            &file_slice.text,
                            &(local_name_start..local_name_end),
                        );
                        let range = byte_span_to_range(
                            &file_slice.text,
                            &(local_decl_start..local_decl_end),
                        );

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: world.name.node.clone(),
                            detail: Some("world".to_string()),
                            kind: SymbolKind::NAMESPACE,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range,
                            children: None,
                        });
                    }
                }
            }
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let state = self.state.read().await;
        let ast = match &state.ast {
            Some(a) => a,
            None => return Ok(None),
        };

        let entity_name = match entity_at_cursor(&state, &uri, pos) {
            Some(n) => n,
            None => return Ok(None),
        };

        let all_refs = find_all_entity_references(ast, &entity_name);

        let mut locations = Vec::new();
        for (span, is_def) in &all_refs {
            if !include_declaration && *is_def {
                continue;
            }
            if let Some(loc) = global_span_to_location(&state.slices, span) {
                locations.push(loc);
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let pos = params.position;

        let state = self.state.read().await;

        let entity_name = match entity_at_cursor(&state, &uri, pos) {
            Some(n) => n,
            None => return Ok(None),
        };

        let text = get_file_text(&state, &uri);
        let offset = position_to_byte_offset(&text, pos);
        let text_lower = text.to_lowercase();
        let name_lower = entity_name.to_lowercase();

        let mut search_from = 0;
        while let Some(found) = text_lower[search_from..].find(&name_lower) {
            let start = search_from + found;
            let end = start + entity_name.len();
            if offset >= start && offset <= end {
                let range = byte_span_to_range(&text, &(start..end));
                return Ok(Some(PrepareRenameResponse::Range(range)));
            }
            search_from = start + 1;
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let new_name = params.new_name;

        let state = self.state.read().await;
        let ast = match &state.ast {
            Some(a) => a,
            None => return Ok(None),
        };

        let entity_name = match entity_at_cursor(&state, &uri, pos) {
            Some(n) => n,
            None => return Ok(None),
        };

        let all_refs = find_all_entity_references(ast, &entity_name);

        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        for (span, _) in &all_refs {
            if let Some(loc) = global_span_to_location(&state.slices, span) {
                changes.entry(loc.uri).or_default().push(TextEdit {
                    range: loc.range,
                    new_text: new_name.clone(),
                });
            }
        }

        if changes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }))
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let state = self.state.read().await;

        let text = get_file_text(&state, &uri);
        if text.is_empty() {
            return Ok(None);
        }

        let (tokens, _) = lexer::lex(&text);
        let entity_spans = collect_entity_name_spans(&text, &state.entity_names);

        let mut semantic_tokens = Vec::new();
        let mut prev_line = 0u32;
        let mut prev_start = 0u32;

        for (i, (token, span)) in tokens.iter().enumerate() {
            if matches!(token, Token::Newline) {
                continue;
            }

            let token_type = if is_in_entity_span(&entity_spans, span) {
                Some(1) // TYPE
            } else {
                classify_token(token, tokens.get(i + 1).map(|(t, _)| t))
            };

            let token_type = match token_type {
                Some(t) => t,
                None => continue,
            };

            let pos = byte_offset_to_position(&text, span.start);
            let length = (span.end - span.start) as u32;

            let delta_line = pos.line - prev_line;
            let delta_start = if delta_line == 0 {
                pos.character - prev_start
            } else {
                pos.character
            };

            semantic_tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = pos.line;
            prev_start = pos.character;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let mut actions = Vec::new();

        for diag in &params.context.diagnostics {
            if let Some(entity_name) = extract_entity_name_from_diagnostic(&diag.message) {
                let state = self.state.read().await;
                let text = get_file_text(&state, &uri);
                drop(state);

                let stub = format!(
                    "\n{entity_name} is a character {{\n    -- TODO: fill in details\n}}\n"
                );

                let end_pos = byte_offset_to_position(&text, text.len());
                let edit = TextEdit {
                    range: Range {
                        start: end_pos,
                        end: end_pos,
                    },
                    new_text: stub,
                };

                let mut changes = HashMap::new();
                changes.insert(uri.clone(), vec![edit]);

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Create entity \"{entity_name}\""),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diag.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

/// Get a file's text from open docs or disk.
fn get_file_text(state: &WorkspaceState, uri: &Url) -> String {
    if let Some(text) = state.open_docs.get(uri) {
        return text.clone();
    }
    if let Ok(path) = uri.to_file_path()
        && let Ok(text) = std::fs::read_to_string(path)
    {
        return text;
    }
    String::new()
}

fn find_word_at_offset(text: &str, offset: usize) -> &str {
    if offset >= text.len() {
        return "";
    }

    let bytes = text.as_bytes();

    let mut start = offset;
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = offset;
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }

    &text[start..end]
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- byte_offset_to_position --

    #[test]
    fn offset_to_position_start() {
        let pos = byte_offset_to_position("hello\nworld\nfoo", 0);
        assert_eq!(pos, Position::new(0, 0));
    }

    #[test]
    fn offset_to_position_first_line() {
        let pos = byte_offset_to_position("hello\nworld\nfoo", 3);
        assert_eq!(pos, Position::new(0, 3));
    }

    #[test]
    fn offset_to_position_second_line() {
        // "hello\n" = 6 bytes, so offset 8 is line 1, char 2
        let pos = byte_offset_to_position("hello\nworld\nfoo", 8);
        assert_eq!(pos, Position::new(1, 2));
    }

    #[test]
    fn offset_to_position_end_of_text() {
        // offset == text.len() should clamp to last position
        let text = "hello\nworld\nfoo";
        let pos = byte_offset_to_position(text, text.len());
        assert_eq!(pos, Position::new(2, 3));
    }

    #[test]
    fn offset_to_position_beyond_eof() {
        let text = "hello\nworld\nfoo";
        let pos = byte_offset_to_position(text, 1000);
        // Clamped to text.len(), same as end_of_text
        assert_eq!(pos, Position::new(2, 3));
    }

    // -- position_to_byte_offset --

    #[test]
    fn position_to_offset_start() {
        let offset = position_to_byte_offset("hello\nworld\nfoo", Position::new(0, 0));
        assert_eq!(offset, 0);
    }

    #[test]
    fn position_to_offset_within_line() {
        let offset = position_to_byte_offset("hello\nworld\nfoo", Position::new(0, 3));
        assert_eq!(offset, 3);
    }

    #[test]
    fn position_to_offset_second_line() {
        // line 1, char 2 → "hello\n" (6) + 2 = 8
        let offset = position_to_byte_offset("hello\nworld\nfoo", Position::new(1, 2));
        assert_eq!(offset, 8);
    }

    #[test]
    fn position_to_offset_beyond_eof() {
        let text = "hello\nworld\nfoo";
        let offset = position_to_byte_offset(text, Position::new(99, 0));
        assert_eq!(offset, text.len());
    }

    // -- Round-trip --

    #[test]
    fn offset_position_round_trip() {
        let text = "hello\nworld\nfoo bar";
        for offset in [0, 3, 5, 6, 8, 12, 15, text.len()] {
            let pos = byte_offset_to_position(text, offset);
            let back = position_to_byte_offset(text, pos);
            assert_eq!(
                back,
                offset.min(text.len()),
                "round-trip failed for offset {offset}"
            );
        }
    }

    #[test]
    fn position_offset_round_trip() {
        let text = "hello\nworld\nfoo";
        let positions = [
            Position::new(0, 0),
            Position::new(0, 3),
            Position::new(1, 0),
            Position::new(1, 4),
            Position::new(2, 2),
        ];
        for pos in positions {
            let offset = position_to_byte_offset(text, pos);
            let back = byte_offset_to_position(text, offset);
            assert_eq!(
                back, pos,
                "round-trip failed for position ({}, {})",
                pos.line, pos.character
            );
        }
    }

    // -- byte_span_to_range --

    #[test]
    fn span_to_range_single_line() {
        let range = byte_span_to_range("hello world", &(0..5));
        assert_eq!(range.start, Position::new(0, 0));
        assert_eq!(range.end, Position::new(0, 5));
    }

    #[test]
    fn span_to_range_multiline() {
        // Span from "hel" on line 0 to "wo" on line 1
        let range = byte_span_to_range("hello\nworld", &(3..8));
        assert_eq!(range.start, Position::new(0, 3));
        assert_eq!(range.end, Position::new(1, 2));
    }

    #[test]
    fn span_to_range_zero_width() {
        let range = byte_span_to_range("hello", &(3..3));
        assert_eq!(range.start, Position::new(0, 3));
        assert_eq!(range.end, Position::new(0, 3));
    }

    // -- find_word_at_offset --

    #[test]
    fn word_at_middle() {
        assert_eq!(find_word_at_offset("hello world", 2), "hello");
    }

    #[test]
    fn word_at_start_of_word() {
        // Offset 6 is 'w' in "world"
        assert_eq!(find_word_at_offset("hello world", 6), "world");
    }

    #[test]
    fn word_at_end_of_word() {
        // Offset 10 is 'd' in "world"
        assert_eq!(find_word_at_offset("hello world", 10), "world");
    }

    #[test]
    fn word_at_punctuation() {
        // Offset on '{' with no preceding word char
        assert_eq!(find_word_at_offset("{ hello }", 0), "");
    }

    #[test]
    fn word_at_out_of_bounds() {
        assert_eq!(find_word_at_offset("hello", 100), "");
        assert_eq!(find_word_at_offset("", 0), "");
    }

    #[test]
    fn word_includes_underscores_and_digits() {
        assert_eq!(find_word_at_offset("foo_bar2 baz", 3), "foo_bar2");
    }

    // -- is_word_char --

    #[test]
    fn word_char_classification() {
        assert!(is_word_char(b'a'));
        assert!(is_word_char(b'Z'));
        assert!(is_word_char(b'0'));
        assert!(is_word_char(b'_'));
        assert!(!is_word_char(b' '));
        assert!(!is_word_char(b'{'));
        assert!(!is_word_char(b'\n'));
        assert!(!is_word_char(b'-'));
    }

    // -- find_definition_offset --

    #[test]
    fn find_def_is_a() {
        let text = "Kael is a character { species human }";
        assert_eq!(find_definition_offset(text, "Kael"), Some(0));
    }

    #[test]
    fn find_def_is_an() {
        let text = "the Sundering is an event { type cataclysm }";
        assert_eq!(find_definition_offset(text, "the Sundering"), Some(0));
    }

    #[test]
    fn find_def_case_insensitive() {
        let text = "KAEL is a character { species human }";
        assert_eq!(find_definition_offset(text, "kael"), Some(0));
    }

    #[test]
    fn find_def_reference_before_definition() {
        // "Kael" appears as a reference first, then as a definition
        let text = "member of Kael\nKael is a character { species human }";
        // Definition starts at offset 15 (after "member of Kael\n")
        assert_eq!(find_definition_offset(text, "Kael"), Some(15));
    }

    #[test]
    fn find_def_not_defined() {
        let text = "just some random text without entities";
        assert_eq!(find_definition_offset(text, "Kael"), None);
    }

    #[test]
    fn find_def_fallback_to_first_occurrence() {
        // Name appears but never as "Name is a/an ..." — falls back to first occurrence
        let text = "allied with Kael\nlocated at Kael";
        assert_eq!(find_definition_offset(text, "Kael"), Some(12));
    }

    // -- find_slice_for_offset / find_slice_for_span --

    fn test_slices() -> Vec<FileSlice> {
        vec![
            FileSlice {
                uri: Url::parse("file:///a.ww").unwrap(),
                offset: 0,
                len: 50,
                text: String::new(),
            },
            FileSlice {
                uri: Url::parse("file:///b.ww").unwrap(),
                offset: 51, // 1-byte gap for newline
                len: 30,
                text: String::new(),
            },
        ]
    }

    #[test]
    fn slice_for_offset_first_file() {
        let slices = test_slices();
        let s = find_slice_for_offset(&slices, 25).unwrap();
        assert_eq!(s.uri.path(), "/a.ww");
    }

    #[test]
    fn slice_for_offset_second_file() {
        let slices = test_slices();
        let s = find_slice_for_offset(&slices, 60).unwrap();
        assert_eq!(s.uri.path(), "/b.ww");
    }

    #[test]
    fn slice_for_offset_gap() {
        let slices = test_slices();
        // Offset 50 is in the newline gap between files
        assert!(find_slice_for_offset(&slices, 50).is_none());
    }

    #[test]
    fn slice_for_span_delegates() {
        let slices = test_slices();
        let s = find_slice_for_span(&slices, &(55..65)).unwrap();
        assert_eq!(s.uri.path(), "/b.ww");
    }

    // -- collect_ww_files --

    #[test]
    fn collect_discovers_ww_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("world.ww"), "world \"Test\" {}").unwrap();
        std::fs::write(dir.path().join("chars.ww"), "Kael is a character {}").unwrap();
        std::fs::write(dir.path().join("readme.txt"), "not a ww file").unwrap();

        let mut files = Vec::new();
        collect_ww_files(&dir.path().to_path_buf(), &mut files);
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|p| p.extension().unwrap() == "ww"));
    }

    #[test]
    fn collect_skips_hidden_and_target() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("world.ww"), "").unwrap();

        let hidden = dir.path().join(".hidden");
        std::fs::create_dir(&hidden).unwrap();
        std::fs::write(hidden.join("secret.ww"), "").unwrap();

        let target = dir.path().join("target");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("build.ww"), "").unwrap();

        let nm = dir.path().join("node_modules");
        std::fs::create_dir(&nm).unwrap();
        std::fs::write(nm.join("dep.ww"), "").unwrap();

        let mut files = Vec::new();
        collect_ww_files(&dir.path().to_path_buf(), &mut files);
        assert_eq!(files.len(), 1, "should only find root world.ww");
    }

    #[test]
    fn collect_recurses_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("root.ww"), "").unwrap();

        let sub = dir.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("nested.ww"), "").unwrap();

        let mut files = Vec::new();
        collect_ww_files(&dir.path().to_path_buf(), &mut files);
        assert_eq!(files.len(), 2);
    }

    // -- find_all_entity_references --

    fn parse_source(source: &str) -> SourceFile {
        let (tokens, lex_errors) = lexer::lex(source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        parser::parse(&tokens).expect("parse error")
    }

    #[test]
    fn find_refs_definition_and_target() {
        let source = "Kael is a character {\n    allied with Elara\n}\nElara is a character {\n    rival of Kael\n}";
        let ast = parse_source(source);
        let refs = find_all_entity_references(&ast, "Kael");

        assert_eq!(refs.len(), 2);
        assert!(refs[0].1, "first should be definition");
        assert!(!refs[1].1, "second should be reference");
        assert_eq!(&source[refs[0].0.clone()], "Kael");
        assert_eq!(&source[refs[1].0.clone()], "Kael");
    }

    #[test]
    fn find_refs_case_insensitive() {
        let source = "KAEL is a character {}\nElara is a character {\n    rival of kael\n}";
        let ast = parse_source(source);
        let refs = find_all_entity_references(&ast, "Kael");

        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn find_refs_involving_list() {
        let source = "Kael is a character {}\nElara is a character {}\nthe Battle is an event {\n    involving [Kael, Elara]\n}";
        let ast = parse_source(source);
        let refs = find_all_entity_references(&ast, "Kael");

        assert_eq!(refs.len(), 2);
        assert!(refs[0].1, "first should be definition");
        assert!(!refs[1].1, "second should be involving reference");
    }

    // -- global_span_to_location --

    fn test_slices_with_text() -> Vec<FileSlice> {
        vec![
            FileSlice {
                uri: Url::parse("file:///a.ww").unwrap(),
                offset: 0,
                len: 10,
                text: "0123456789".to_string(),
            },
            FileSlice {
                uri: Url::parse("file:///b.ww").unwrap(),
                offset: 11,
                len: 10,
                text: "abcdefghij".to_string(),
            },
        ]
    }

    #[test]
    fn global_span_to_location_first_file() {
        let slices = test_slices_with_text();
        let loc = global_span_to_location(&slices, &(2..5)).unwrap();
        assert_eq!(loc.uri.path(), "/a.ww");
        assert_eq!(loc.range.start, Position::new(0, 2));
        assert_eq!(loc.range.end, Position::new(0, 5));
    }

    #[test]
    fn global_span_to_location_second_file() {
        let slices = test_slices_with_text();
        let loc = global_span_to_location(&slices, &(13..16)).unwrap();
        assert_eq!(loc.uri.path(), "/b.ww");
        // Offset 13 global = 13-11 = 2 local
        assert_eq!(loc.range.start, Position::new(0, 2));
        assert_eq!(loc.range.end, Position::new(0, 5));
    }

    // -- entity_kind_to_symbol_kind --

    #[test]
    fn entity_kind_symbol_mapping() {
        assert_eq!(entity_kind_to_symbol_kind("character"), SymbolKind::CLASS);
        assert_eq!(
            entity_kind_to_symbol_kind("location"),
            SymbolKind::NAMESPACE
        );
        assert_eq!(
            entity_kind_to_symbol_kind("fortress"),
            SymbolKind::NAMESPACE
        );
        assert_eq!(entity_kind_to_symbol_kind("faction"), SymbolKind::STRUCT);
        assert_eq!(entity_kind_to_symbol_kind("event"), SymbolKind::EVENT);
        assert_eq!(entity_kind_to_symbol_kind("item"), SymbolKind::OBJECT);
        assert_eq!(entity_kind_to_symbol_kind("lore"), SymbolKind::FILE);
        assert_eq!(entity_kind_to_symbol_kind("starship"), SymbolKind::VARIABLE);
    }

    // -- classify_token --

    #[test]
    fn classify_string_token() {
        assert_eq!(classify_token(&Token::Str("hello".into()), None), Some(3));
    }

    #[test]
    fn classify_number_token() {
        assert_eq!(classify_token(&Token::Integer(42), None), Some(4));
    }

    #[test]
    fn classify_brace_token() {
        assert_eq!(classify_token(&Token::LBrace, None), Some(6));
    }

    #[test]
    fn classify_keyword_token() {
        assert_eq!(classify_token(&Token::Word("world".into()), None), Some(0));
    }

    #[test]
    fn classify_property_key_token() {
        let next = Token::Word("human".into());
        assert_eq!(
            classify_token(&Token::Word("species".into()), Some(&next)),
            Some(2)
        );
    }

    // -- detect_completion_context --

    #[test]
    fn detect_context_entity_kind() {
        assert!(matches!(
            detect_completion_context("Kael is a "),
            WwCompletionCtx::EntityKind
        ));
    }

    #[test]
    fn detect_context_relationship_target() {
        assert!(matches!(
            detect_completion_context("    member of "),
            WwCompletionCtx::RelationshipTarget
        ));
    }

    #[test]
    fn detect_context_entity_body() {
        assert!(matches!(
            detect_completion_context("    "),
            WwCompletionCtx::EntityBody
        ));
    }

    #[test]
    fn detect_context_default() {
        assert!(matches!(
            detect_completion_context("Kael"),
            WwCompletionCtx::Default
        ));
    }

    // -- extract_entity_name_from_diagnostic --

    #[test]
    fn extract_entity_name_match() {
        assert_eq!(
            extract_entity_name_from_diagnostic("undefined entity: \"Kael Stormborn\""),
            Some("Kael Stormborn")
        );
    }

    #[test]
    fn extract_entity_name_no_match() {
        assert_eq!(
            extract_entity_name_from_diagnostic("some other error message"),
            None
        );
    }

    // -- source hash (incremental compilation) --

    #[test]
    fn source_hash_different() {
        let hash1 = {
            let mut h = DefaultHasher::new();
            "hello".hash(&mut h);
            h.finish()
        };
        let hash2 = {
            let mut h = DefaultHasher::new();
            "world".hash(&mut h);
            h.finish()
        };
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn source_hash_same() {
        let hash1 = {
            let mut h = DefaultHasher::new();
            "hello world".hash(&mut h);
            h.finish()
        };
        let hash2 = {
            let mut h = DefaultHasher::new();
            "hello world".hash(&mut h);
            h.finish()
        };
        assert_eq!(hash1, hash2);
    }
}
