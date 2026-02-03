use std::path::Path;

use ww_core::EntityKind;

pub fn run(dir: &Path) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    // Print summary
    let counts = world.entity_counts_by_kind();
    println!("  Compiled '{}' successfully.", world.meta.name);
    println!();
    println!("  {} entities, {} relationships", world.entity_count(), world.relationship_count());

    if !counts.is_empty() {
        let mut sorted: Vec<_> = counts.iter().collect();
        sorted.sort_by_key(|(k, _)| kind_sort_order(k));
        for (kind, count) in sorted {
            println!("    {count:>4} {kind}");
        }
    }

    Ok(())
}

fn kind_sort_order(kind: &EntityKind) -> u8 {
    match kind {
        EntityKind::Location => 0,
        EntityKind::Character => 1,
        EntityKind::Faction => 2,
        EntityKind::Event => 3,
        EntityKind::Item => 4,
        EntityKind::Lore => 5,
        EntityKind::Custom(_) => 6,
    }
}
