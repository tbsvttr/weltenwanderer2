//! CLI frontend for the Weltenwanderer world-building engine.

mod commands;
mod tui;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ww",
    about = "Weltenwanderer — a creative engine for world building",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new world directory with template .ww files
    Init {
        /// Name of the world to create
        name: String,
    },

    /// Compile all .ww files and report diagnostics
    Build {
        /// Directory containing .ww files (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Validate .ww files without full compilation output
    Check {
        /// Directory containing .ww files (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// List entities in the compiled world
    List {
        /// Filter by entity kind (e.g. character, location, faction)
        kind: Option<String>,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Show detailed information about an entity
    Show {
        /// Entity name (case-insensitive)
        name: String,

        /// Also show relationships
        #[arg(short, long)]
        relationships: bool,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Search entities by name, description, or tags
    Search {
        /// Search query
        query: String,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Display an ASCII relationship graph
    Graph {
        /// Focus on a specific entity
        #[arg(short, long)]
        focus: Option<String>,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Display events in chronological order
    Timeline {
        /// Start year (inclusive)
        #[arg(long)]
        from: Option<i64>,

        /// End year (inclusive)
        #[arg(long)]
        to: Option<i64>,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Export the world to a different format
    Export {
        /// Output format: json, markdown, html
        format: String,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Run a tick-based simulation of the world
    Simulate {
        /// Number of ticks to simulate (default: 24 = one day at 1 hour/tick)
        #[arg(short, long, default_value = "24")]
        ticks: u64,

        /// RNG seed for deterministic simulation
        #[arg(short, long, default_value = "42")]
        seed: u64,

        /// In-world hours per tick
        #[arg(long, default_value = "1.0")]
        speed: f64,

        /// Show all events (not just summary)
        #[arg(short, long)]
        verbose: bool,

        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Launch interactive TUI world explorer
    Tui {
        /// Directory containing .ww files
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Start the Language Server Protocol server (for IDE integration)
    Lsp,

    /// Generate a DSL stub for a new entity
    New {
        /// Entity kind (e.g. character, location, faction)
        kind: String,

        /// Entity name
        name: String,

        /// File to append to (default: `<kind>s.ww`)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => commands::init::run(&name),
        Commands::Build { dir } => commands::build::run(&dir),
        Commands::Check { dir } => commands::check::run(&dir),
        Commands::List { kind, tag, dir } => {
            commands::list::run(&dir, kind.as_deref(), tag.as_deref())
        }
        Commands::Show {
            name,
            relationships,
            dir,
        } => commands::show::run(&dir, &name, relationships),
        Commands::Search { query, dir } => commands::search::run(&dir, &query),
        Commands::Graph { focus, dir } => commands::graph::run(&dir, focus.as_deref()),
        Commands::Timeline { from, to, dir } => commands::timeline::run(&dir, from, to),
        Commands::Export {
            format,
            output,
            dir,
        } => commands::export::run(&dir, &format, output.as_deref()),
        Commands::Simulate {
            ticks,
            seed,
            speed,
            verbose,
            dir,
        } => commands::simulate::run(&dir, ticks, seed, speed, verbose),
        Commands::Tui { dir } => commands::compile_dir_for_tui(&dir).and_then(tui::run),
        Commands::Lsp => {
            // Exec the separate ww-lsp binary
            let status = std::process::Command::new("ww-lsp")
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(s) if s.success() => Ok(()),
                Ok(s) => Err(format!("ww-lsp exited with {s}")),
                Err(_) => Err(
                    "ww-lsp binary not found. Install it with: cargo install --path crates/ww-lsp"
                        .into(),
                ),
            }
        }
        Commands::New { kind, name, file } => commands::new::run(&kind, &name, file.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
