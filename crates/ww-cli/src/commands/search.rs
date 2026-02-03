use std::path::Path;

pub fn run(dir: &Path, query: &str) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    let results = world.search(query);

    if results.is_empty() {
        println!("  No results for \"{}\".", query);
        return Ok(());
    }

    println!("  {} results for \"{}\":", results.len(), query);
    println!();

    for entity in &results {
        let kind_str = if let Some(subtype) = entity.location_subtype() {
            format!("{} ({})", entity.kind, subtype)
        } else {
            entity.kind.to_string()
        };

        println!("  {} [{}]", entity.name, kind_str);

        if !entity.description.is_empty() {
            let preview = if entity.description.len() > 80 {
                format!("{}...", &entity.description[..77])
            } else {
                entity.description.clone()
            };
            println!("    {}", preview.trim());
        }
    }

    Ok(())
}
