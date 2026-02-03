use std::path::Path;

pub fn run(dir: &Path) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    println!("  All checks passed for '{}'.", world.meta.name);
    println!("  {} entities, {} relationships", world.entity_count(), world.relationship_count());

    Ok(())
}
