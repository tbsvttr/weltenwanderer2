//! Launch the ww-tui standalone binary for interactive fiction play.

use std::path::Path;

/// Launch the ww-tui standalone binary for interactive fiction play.
pub fn run(dir: &Path) -> Result<(), String> {
    let status = std::process::Command::new("ww-tui")
        .arg("--world")
        .arg(dir)
        .arg("--tab")
        .arg("play")
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("ww-tui exited with {s}")),
        Err(_) => {
            Err("ww-tui binary not found. Install with: cargo install --path crates/ww-tui".into())
        }
    }
}
