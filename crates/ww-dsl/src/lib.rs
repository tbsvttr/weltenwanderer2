//! DSL lexer, parser, and compiler for Weltenwanderer world files.
//!
//! `.ww` files are the source of truth. They're human-readable, git-friendly,
//! and designed to feel natural while remaining strictly parseable.
//!
//! ### Entity Declarations
//!
//! Every entity follows the pattern `<Name> is a <kind> { ... }`:
//!
//! ```ww
//! -- This is a comment
//!
//! world "The Iron Kingdoms" {
//!     genre "high fantasy"
//!     setting "A shattered continent rebuilding after the Sundering"
//! }
//!
//! the Iron Citadel is a fortress {
//!     climate arid
//!     population 45000
//!     north to the Ashlands
//!
//!     """
//!     An ancient fortress carved from a single mountain of iron ore.
//!     Its walls have never been breached in three thousand years.
//!     """
//! }
//!
//! Kael Stormborn is a character {
//!     species human
//!     occupation knight
//!     status alive
//!     traits [brave, stubborn, loyal]
//!     member of the Order of Dawn
//!     located at the Iron Citadel
//!     allied with Elara Nightwhisper
//! }
//!
//! the Order of Dawn is a faction {
//!     type military_order
//!     led by Kael Stormborn
//!     based at the Iron Citadel
//!     values [honor, duty, sacrifice]
//! }
//!
//! the Great Sundering is an event {
//!     date year -1247, month 3, day 15, era "Age of Ruin"
//!     type cataclysm
//!     involving [Kael Stormborn, the Order of Dawn]
//!
//!     """
//!     The day the world broke.
//!     """
//! }
//!
//! the Blade of First Light is an item {
//!     type weapon
//!     rarity legendary
//!     owned by Kael Stormborn
//! }
//!
//! the Prophecy of Renewal is lore {
//!     type prophecy
//!     source "Ancient Dwarven Tablets"
//!     references [the Great Sundering, the Iron Citadel]
//! }
//! ```
//!
//! ### Syntax Reference
//!
//! | Pattern | Meaning |
//! |---|---|
//! | `<Name> is a <kind> { ... }` | Entity declaration |
//! | `<Name> is <kind> { ... }` | Entity declaration (no article) |
//! | `<key> <value>` | Property assignment |
//! | `<key> [a, b, c]` | List property |
//! | `<direction> to <Entity>` | Exit/connection (north, south, east, west, up, down) |
//! | `member of <Entity>` | Relationship: membership |
//! | `located at <Entity>` | Relationship: location |
//! | `allied with <Entity>` | Relationship: alliance |
//! | `rival of <Entity>` | Relationship: rivalry |
//! | `led by <Entity>` | Relationship: leadership |
//! | `owned by <Entity>` | Relationship: ownership |
//! | `based at <Entity>` | Relationship: headquarters |
//! | `in <Entity>` | Relationship: containment |
//! | `involving [<Entity>, ...]` | Relationship: participation |
//! | `references [<Entity>, ...]` | Relationship: reference |
//! | `caused by <Entity>` | Relationship: causation |
//! | `date year N, month N, day N, era "E"` | Date (year required, month/day/era optional) |
//! | `"""..."""` | Multiline description (Markdown) |
//! | `-- comment` | Line comment |
//! | `"string"` | Quoted string value |
//! | `45_000`, `-1247` | Numbers (Rust-style underscores allowed) |
//!
//! ### Entity Kinds
//!
//! Built-in: `location`, `character`, `faction`, `event`, `item`, `lore`
//!
//! Location subtypes (compiled as `location` with a subtype): `fortress`, `city`,
//! `town`, `village`, `region`, `continent`, `room`, `wilderness`, `dungeon`,
//! `building`, `landmark`, `plane`
//!
//! Any unrecognized kind becomes a custom type.
//!
//! ### Multi-File Support
//!
//! The compiler reads all `.ww` files in a directory. File boundaries don't
//! matter — entities can reference each other across files. See `iron-kingdoms/`
//! for an example world split across multiple files.

/// Abstract syntax tree types produced by the parser.
pub mod ast;
/// Compiler that transforms AST declarations into a `ww_core::World`.
pub mod compiler;
/// Diagnostic types and pretty-printing for errors and warnings.
pub mod diagnostics;
/// Tokenizer (lexer) for `.ww` source files.
pub mod lexer;
/// Chumsky-based parser that turns token streams into an AST.
pub mod parser;
/// Cross-file name resolver and source-map utilities.
pub mod resolver;

use std::path::Path;

/// Re-export of [`compiler::CompileResult`] for convenience.
pub use compiler::CompileResult;
/// Re-export of [`diagnostics::Diagnostic`] for convenience.
pub use diagnostics::Diagnostic;
/// Re-export of [`resolver::SourceMap`] for convenience.
pub use resolver::SourceMap;

/// An input file for multi-file compilation.
pub struct InputFile {
    /// Display name (e.g., "characters.ww").
    pub name: String,
    /// Source text content.
    pub text: String,
}

/// Compile a single source string into a World.
pub fn compile_source(source: &str) -> CompileResult {
    let source_map = resolver::SourceMap::single(source.len());
    compile_with_source_map(source, source_map)
}

/// Compile multiple named source files into a single World.
///
/// This is the preferred API when file contents are already in memory
/// (e.g., from the LSP). The returned [`CompileResult`] includes a
/// [`SourceMap`] that maps diagnostic spans back to individual files.
pub fn compile_files(files: &[InputFile]) -> CompileResult {
    let mut concatenated = String::new();
    let mut source_map = resolver::SourceMap::new();

    for file in files {
        if !concatenated.is_empty() {
            concatenated.push('\n');
        }
        let offset = concatenated.len();
        source_map.add_file(file.name.clone(), offset, file.text.len());
        concatenated.push_str(&file.text);
    }

    if concatenated.is_empty() {
        return CompileResult {
            world: ww_core::World::new(ww_core::WorldMeta::new("Empty")),
            diagnostics: vec![Diagnostic::error(0..0, "no source files provided")],
            source_map,
        };
    }

    compile_with_source_map(&concatenated, source_map)
}

/// Compile all `.ww` files in a directory into a single World.
pub fn compile_dir(dir: &Path) -> CompileResult {
    let mut sources = String::new();
    let mut source_map = resolver::SourceMap::new();

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "ww"))
            .collect(),
        Err(e) => {
            return CompileResult {
                world: ww_core::World::new(ww_core::WorldMeta::new("Error")),
                diagnostics: vec![Diagnostic::error(
                    0..0,
                    format!("cannot read directory: {e}"),
                )],
                source_map,
            };
        }
    };

    // Sort for deterministic ordering
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        match std::fs::read_to_string(entry.path()) {
            Ok(content) => {
                if !sources.is_empty() {
                    sources.push('\n');
                }
                let offset = sources.len();
                let len = content.len();
                let file_name = entry
                    .path()
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| entry.path().display().to_string());
                source_map.add_file(file_name, offset, len);
                sources.push_str(&content);
            }
            Err(e) => {
                return CompileResult {
                    world: ww_core::World::new(ww_core::WorldMeta::new("Error")),
                    diagnostics: vec![Diagnostic::error(
                        0..0,
                        format!("cannot read {}: {e}", entry.path().display()),
                    )],
                    source_map,
                };
            }
        }
    }

    if sources.is_empty() {
        return CompileResult {
            world: ww_core::World::new(ww_core::WorldMeta::new("Empty")),
            diagnostics: vec![Diagnostic::error(
                0..0,
                format!("no .ww files found in {}", dir.display()),
            )],
            source_map,
        };
    }

    compile_with_source_map(&sources, source_map)
}

/// Internal: compile a source string with an explicit SourceMap.
fn compile_with_source_map(source: &str, source_map: resolver::SourceMap) -> CompileResult {
    let (tokens, lex_errors) = lexer::lex(source);

    let mut diagnostics: Vec<Diagnostic> = lex_errors
        .into_iter()
        .map(|e| Diagnostic::error(e.span, e.message))
        .collect();

    let ast = match parser::parse(&tokens) {
        Ok(ast) => ast,
        Err(parse_errors) => {
            diagnostics.extend(
                parse_errors
                    .into_iter()
                    .map(|e| Diagnostic::error(e.span, e.message)),
            );
            return CompileResult {
                world: ww_core::World::new(ww_core::WorldMeta::new("Error")),
                diagnostics,
                source_map,
            };
        }
    };

    let resolver = resolver::Resolver::resolve(&ast, &source_map);
    let mut result = compiler::compile(&ast, &resolver, source_map);
    // Prepend lex/parse errors before resolve/compile diagnostics
    diagnostics.append(&mut result.diagnostics);
    result.diagnostics = diagnostics;
    result
}
