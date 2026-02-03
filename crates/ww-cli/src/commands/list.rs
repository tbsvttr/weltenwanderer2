use std::path::Path;

use comfy_table::{ContentArrangement, Table};
use ww_core::entity::EntityKind;

pub fn run(dir: &Path, kind: Option<&str>, tag: Option<&str>) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    let mut query = world.query();

    if let Some(kind_str) = kind {
        let (entity_kind, _) = EntityKind::parse(kind_str);
        query = query.kind(entity_kind);
    }

    if let Some(tag_str) = tag {
        query = query.tag(tag_str);
    }

    let results = query.execute();

    if results.is_empty() {
        println!("  No entities found.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Name", "Kind", "Description"]);

    for entity in &results {
        let desc = if entity.description.len() > 60 {
            format!("{}...", &entity.description[..57])
        } else if entity.description.is_empty() {
            "—".to_string()
        } else {
            entity.description.clone()
        };

        let kind_str = if let Some(subtype) = entity.location_subtype() {
            format!("{} ({})", entity.kind, subtype)
        } else {
            entity.kind.to_string()
        };

        table.add_row(vec![&entity.name, &kind_str, &desc]);
    }

    println!("{table}");
    println!();
    println!("  {} entities", results.len());

    Ok(())
}
