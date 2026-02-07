use std::io::{self, BufRead, Write};
use std::path::Path;

use colored::Colorize;

use ww_fiction::FictionSession;

pub fn run(dir: &Path) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    let mut session =
        FictionSession::new(world).map_err(|e| format!("failed to start session: {e}"))?;

    println!("  {} '{}'", "Playing".bold(), session.world().meta.name);
    println!("  Type 'help' for commands, 'quit' to exit.\n");

    // Show initial location
    let initial = session.process("look").map_err(|e| format!("{e}"))?;
    println!("{initial}\n");

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
                println!("{output}\n");
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
