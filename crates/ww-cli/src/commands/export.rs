use std::path::Path;

use ww_core::World;
use ww_core::entity::EntityKind;

pub fn run(dir: &Path, format: &str, output: Option<&Path>) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    let content = match format {
        "json" => export_json(&world)?,
        "markdown" | "md" => export_markdown(&world),
        "html" => export_html(&world),
        _ => {
            return Err(format!(
                "unsupported format: \"{format}\". Use: json, markdown, html"
            ));
        }
    };

    if let Some(path) = output {
        std::fs::write(path, &content)
            .map_err(|e| format!("cannot write to {}: {e}", path.display()))?;
        println!("  Exported to {}", path.display());
    } else {
        print!("{content}");
    }

    Ok(())
}

fn export_json(world: &World) -> Result<String, String> {
    // Build a serializable structure
    let entities: Vec<_> = world.all_entities().collect();
    let relationships: Vec<_> = world.all_relationships().collect();

    let export = serde_json::json!({
        "world": {
            "name": world.meta.name,
            "description": world.meta.description,
            "genre": world.meta.genre,
            "setting": world.meta.setting,
        },
        "entities": entities,
        "relationships": relationships,
    });

    serde_json::to_string_pretty(&export).map_err(|e| format!("JSON serialization error: {e}"))
}

fn export_markdown(world: &World) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", world.meta.name));

    if !world.meta.description.is_empty() {
        out.push_str(&format!("{}\n\n", world.meta.description));
    }
    if let Some(ref genre) = world.meta.genre {
        out.push_str(&format!("**Genre:** {genre}\n\n"));
    }
    if let Some(ref setting) = world.meta.setting {
        out.push_str(&format!("**Setting:** {setting}\n\n"));
    }

    out.push_str("---\n\n");

    // Group by kind
    let kinds = [
        EntityKind::Location,
        EntityKind::Character,
        EntityKind::Faction,
        EntityKind::Event,
        EntityKind::Item,
        EntityKind::Lore,
    ];

    for kind in &kinds {
        let entities = world.entities_by_kind(kind);
        if entities.is_empty() {
            continue;
        }

        let kind_name = match kind {
            EntityKind::Location => "Locations",
            EntityKind::Character => "Characters",
            EntityKind::Faction => "Factions",
            EntityKind::Event => "Events",
            EntityKind::Item => "Items",
            EntityKind::Lore => "Lore",
            _ => "Other",
        };

        out.push_str(&format!("## {kind_name}\n\n"));

        let mut sorted: Vec<_> = entities;
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        for entity in sorted {
            out.push_str(&format!("### {}\n\n", entity.name));

            if !entity.description.is_empty() {
                out.push_str(&format!("{}\n\n", entity.description.trim()));
            }

            // Properties
            if !entity.properties.is_empty() {
                let mut props: Vec<_> = entity.properties.iter().collect();
                props.sort_by_key(|(k, _)| (*k).clone());
                for (key, value) in props {
                    out.push_str(&format!("- **{key}:** {value}\n"));
                }
                out.push('\n');
            }

            // Relationships
            let rels = world.relationships_of(entity.id);
            if !rels.is_empty() {
                out.push_str("**Relationships:**\n\n");
                for rel in &rels {
                    let other_id = if rel.source == entity.id {
                        rel.target
                    } else {
                        rel.source
                    };
                    let other_name = world.entity_name(other_id);
                    let phrase = rel.kind.as_phrase();
                    out.push_str(&format!("- {phrase} {other_name}\n"));
                }
                out.push('\n');
            }
        }
    }

    out
}

fn export_html(world: &World) -> String {
    // Wrap the markdown in a basic HTML template
    let md = export_markdown(world);

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str(&format!(
        "  <meta charset=\"utf-8\">\n  <title>{}</title>\n",
        world.meta.name
    ));
    html.push_str("  <style>\n");
    html.push_str("    body { font-family: Georgia, serif; max-width: 800px; margin: 2em auto; padding: 0 1em; color: #333; }\n");
    html.push_str("    h1 { border-bottom: 2px solid #666; padding-bottom: 0.3em; }\n");
    html.push_str("    h2 { color: #555; margin-top: 2em; }\n");
    html.push_str("    h3 { color: #444; }\n");
    html.push_str("    hr { border: none; border-top: 1px solid #ccc; margin: 2em 0; }\n");
    html.push_str("    pre { background: #f5f5f5; padding: 1em; overflow-x: auto; }\n");
    html.push_str("  </style>\n</head>\n<body>\n<pre>\n");
    html.push_str(&md);
    html.push_str("</pre>\n</body>\n</html>\n");

    html
}
