use std::collections::HashSet;
use std::path::Path;

use ww_core::World;
use ww_core::entity::EntityId;

pub fn run(dir: &Path, focus: Option<&str>) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    if let Some(focus_name) = focus {
        let entity = world
            .find_by_name(focus_name)
            .ok_or_else(|| format!("entity not found: \"{}\"", focus_name))?;

        println!("  Graph for: {}", entity.name);
        println!();
        render_focused_graph(&world, entity.id);
    } else {
        println!("  Relationship graph for '{}'", world.meta.name);
        println!();
        render_full_graph(&world);
    }

    Ok(())
}

fn render_focused_graph(world: &World, center: EntityId) {
    let center_name = world.entity_name(center);

    let rels = world.relationships_of(center);

    if rels.is_empty() {
        println!("  [{center_name}]");
        println!("    (no relationships)");
        return;
    }

    // Group by relationship kind
    let mut seen_entities: HashSet<EntityId> = HashSet::new();
    seen_entities.insert(center);

    println!("  [{center_name}]");

    for rel in &rels {
        let (other_id, direction, phrase) = if rel.source == center {
            let label = if let Some(ref l) = rel.label {
                format!("{} ({})", rel.kind.as_phrase(), l)
            } else {
                rel.kind.as_phrase().to_string()
            };
            (rel.target, "-->", label)
        } else {
            let label = if let Some(ref l) = rel.label {
                format!("{} ({})", rel.kind.as_phrase(), l)
            } else {
                rel.kind.as_phrase().to_string()
            };
            (rel.source, "<--", label)
        };

        if !seen_entities.insert(other_id) {
            continue; // Skip duplicate edges to the same entity
        }

        let other_name = world.entity_name(other_id);

        println!("    {direction} {phrase} --> [{other_name}]");
    }
}

fn render_full_graph(world: &World) {
    let mut seen_pairs: HashSet<(EntityId, EntityId)> = HashSet::new();

    for rel in world.all_relationships() {
        let (a, b) = (rel.source.0, rel.target.0);
        let pair = if a < b {
            (rel.source, rel.target)
        } else {
            (rel.target, rel.source)
        };

        // For the full graph, show each relationship once
        if !seen_pairs.insert(pair) && rel.bidirectional {
            continue;
        }

        let source_name = world.entity_name(rel.source);
        let target_name = world.entity_name(rel.target);

        let arrow = if rel.bidirectional { "<-->" } else { " -->" };
        let label = if let Some(ref l) = rel.label {
            format!("{} ({})", rel.kind.as_phrase(), l)
        } else {
            rel.kind.as_phrase().to_string()
        };

        println!("  [{source_name}] {arrow} {label} {arrow} [{target_name}]");
    }

    let stats = world.entity_counts_by_kind();
    println!();
    println!(
        "  {} entities, {} relationships",
        world.entity_count(),
        world.relationship_count()
    );
    if !stats.is_empty() {
        let mut sorted: Vec<_> = stats.iter().collect();
        sorted.sort_by_key(|(k, _)| format!("{k}"));
        let summary: Vec<String> = sorted.iter().map(|(k, v)| format!("{v} {k}")).collect();
        println!("  ({})", summary.join(", "));
    }
}
