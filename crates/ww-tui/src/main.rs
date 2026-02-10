//! Standalone TUI binary for Weltenwanderer.

use std::path::PathBuf;
use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ww-tui",
    about = "Terminal UI for Weltenwanderer world building",
    version
)]
struct Args {
    /// Directory containing .ww files
    #[arg(long)]
    world: PathBuf,

    /// Start on a specific tab (explorer, graph, timeline, play, solo, sheet, dice)
    #[arg(long, default_value = "explorer")]
    tab: String,

    /// RNG seed for solo/dice
    #[arg(long, default_value = "42")]
    seed: u64,

    /// Initial chaos factor for solo (1-9)
    #[arg(long, default_value = "5")]
    chaos: u32,
}

fn main() {
    let args = Args::parse();

    let world = match compile_dir(&args.world) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let tab = ww_tui::tabs::TabId::from_name(&args.tab).unwrap_or(ww_tui::tabs::TabId::Explorer);

    let app = ww_tui::app::TuiApp::new(world, tab, args.seed, args.chaos);

    if let Err(e) = ww_tui::terminal::run(app) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

/// Compile all `.ww` files in a directory into a World.
fn compile_dir(dir: &std::path::Path) -> Result<ww_core::World, String> {
    let result = ww_dsl::compile_dir(dir);
    for d in &result.diagnostics {
        eprintln!("{d}");
    }
    if result.has_errors() {
        Err("compilation failed with errors".into())
    } else {
        Ok(result.world)
    }
}
