use std::fs;
use std::path::Path;

pub fn run(name: &str) -> Result<(), String> {
    let dir = Path::new(name);

    if dir.exists() {
        return Err(format!("directory '{}' already exists", name));
    }

    fs::create_dir_all(dir).map_err(|e| format!("cannot create directory: {e}"))?;

    // Create a template world.ww
    let world_content = format!(
        r#"-- World metadata
world "{name}" {{
    genre "fantasy"
    setting "A world of your creation"
}}

-- Add your entities below. For example:
--
-- the Great Hall is a location {{
--     climate temperate
--     population 200
--
--     """
--     A grand hall at the center of the kingdom.
--     """
-- }}
--
-- Kael is a character {{
--     species human
--     occupation knight
--     status alive
--     traits [brave, loyal]
--     located at the Great Hall
-- }}
"#
    );

    fs::write(dir.join("world.ww"), world_content)
        .map_err(|e| format!("cannot write world.ww: {e}"))?;

    println!("Created world '{}' in {}/", name, name);
    println!("  world.ww  — world metadata and template");
    println!();
    println!("Get started:");
    println!("  cd {}", name);
    println!("  # Edit world.ww to define your world");
    println!("  ww build      # Compile and check for errors");
    println!("  ww list        # List all entities");
    println!("  ww show <name> # Show entity details");

    Ok(())
}
