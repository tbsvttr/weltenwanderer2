pub mod build;
pub mod check;
pub mod export;
pub mod graph;
pub mod init;
pub mod list;
pub mod new;
pub mod search;
pub mod show;
pub mod simulate;
pub mod timeline;

use std::path::Path;

use ww_core::World;
use ww_dsl::CompileResult;
use ww_dsl::diagnostics::{Severity, render_diagnostics};

/// Compile for TUI: same as compile_dir but public for main.rs to call.
pub fn compile_dir_for_tui(dir: &Path) -> Result<World, String> {
    compile_dir(dir)
}

/// Compile a directory of .ww files and print diagnostics.
/// Returns the compiled world if there are no errors.
fn compile_dir(dir: &Path) -> Result<World, String> {
    let result = ww_dsl::compile_dir(dir);
    print_diagnostics(&result, dir);

    if result.has_errors() {
        Err("compilation failed with errors".into())
    } else {
        Ok(result.world)
    }
}

/// Print diagnostics to stderr using ariadne.
fn print_diagnostics(result: &CompileResult, dir: &Path) {
    let has_diags = !result.diagnostics.is_empty();
    if !has_diags {
        return;
    }

    // Read all source files to provide context for diagnostics
    let source = read_all_sources(dir);
    let filename = dir.display().to_string();

    let rendered = render_diagnostics(&source, &filename, &result.diagnostics);
    eprint!("{rendered}");

    let errors = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    if errors > 0 {
        eprintln!(
            "  {} error{}, {} warning{}",
            errors,
            if errors == 1 { "" } else { "s" },
            warnings,
            if warnings == 1 { "" } else { "s" },
        );
    } else if warnings > 0 {
        eprintln!(
            "  {} warning{}",
            warnings,
            if warnings == 1 { "" } else { "s" },
        );
    }
}

/// Read and concatenate all .ww source files (for diagnostic rendering).
fn read_all_sources(dir: &Path) -> String {
    let mut sources = String::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "ww"))
            .collect();
        files.sort_by_key(|e| e.path());
        for entry in files {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if !sources.is_empty() {
                    sources.push('\n');
                }
                sources.push_str(&content);
            }
        }
    }
    sources
}
