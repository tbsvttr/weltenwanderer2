pub mod ast;
pub mod compiler;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod resolver;

use std::path::Path;

pub use compiler::CompileResult;
pub use diagnostics::Diagnostic;

/// Compile a single source string into a World.
pub fn compile_source(source: &str) -> CompileResult {
    let (tokens, lex_errors) = lexer::lex(source);

    // Convert lex errors to diagnostics
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
            };
        }
    };

    let mut result = compiler::compile(&ast);
    result.diagnostics.extend(diagnostics);
    result
}

/// Compile all `.ww` files in a directory into a single World.
pub fn compile_dir(dir: &Path) -> CompileResult {
    let mut sources = String::new();

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
                sources.push_str(&content);
            }
            Err(e) => {
                return CompileResult {
                    world: ww_core::World::new(ww_core::WorldMeta::new("Error")),
                    diagnostics: vec![Diagnostic::error(
                        0..0,
                        format!("cannot read {}: {e}", entry.path().display()),
                    )],
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
        };
    }

    compile_source(&sources)
}
