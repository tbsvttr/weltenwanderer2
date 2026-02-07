use std::io::{self, BufRead, Write};
use std::path::Path;

use colored::Colorize;

use ww_solo::{SoloConfig, SoloSession};

pub fn run(dir: &Path, seed: u64, chaos: u32) -> Result<(), String> {
    let world = super::compile_dir(dir)?;
    let config = SoloConfig::default().with_seed(seed).with_chaos(chaos);

    let mut session =
        SoloSession::new(world, config).map_err(|e| format!("failed to start session: {e}"))?;

    println!("  {} Solo TTRPG Session", "Starting".bold());
    println!("  Chaos: {}/9 | Seed: {seed}", chaos.clamp(1, 9));
    println!("  Type 'help' for commands, 'quit' to exit.\n");

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush().map_err(|e| e.to_string())?;

        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Err(e) => return Err(e.to_string()),
            _ => {}
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match session.process(input) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{output}\n");
                }
                if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("q") {
                    break;
                }
            }
            Err(e) => {
                println!("{}\n", e.to_string().yellow());
            }
        }
    }

    Ok(())
}
