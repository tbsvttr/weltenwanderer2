use std::path::Path;

use colored::Colorize;

pub fn run(dir: &Path, name: &str, show_relationships: bool) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    let entity = world
        .find_by_name(name)
        .ok_or_else(|| format!("entity not found: \"{}\"", name))?;

    // Header
    let kind_str = if let Some(subtype) = entity.location_subtype() {
        format!("{} ({})", entity.kind, subtype)
    } else {
        entity.kind.to_string()
    };
    println!("  {} [{}]", entity.name.bold(), kind_str.dimmed());
    println!();

    // Description
    if !entity.description.is_empty() {
        for line in entity.description.lines() {
            println!("  {}", line.trim());
        }
        println!();
    }

    // Component-specific fields
    if let Some(char_comp) = &entity.components.character {
        if let Some(ref species) = char_comp.species {
            println!("  species:    {species}");
        }
        if let Some(ref occupation) = char_comp.occupation {
            println!("  occupation: {occupation}");
        }
        println!("  status:     {:?}", char_comp.status);
        if !char_comp.traits.is_empty() {
            println!("  traits:     {}", char_comp.traits.join(", "));
        }
    }

    if let Some(loc_comp) = &entity.components.location {
        if loc_comp.location_type != "location" {
            println!("  type:       {}", loc_comp.location_type);
        }
        if let Some(ref climate) = loc_comp.climate {
            println!("  climate:    {climate}");
        }
        if let Some(ref terrain) = loc_comp.terrain {
            println!("  terrain:    {terrain}");
        }
        if let Some(population) = loc_comp.population {
            println!("  population: {population}");
        }
    }

    if let Some(faction_comp) = &entity.components.faction {
        if let Some(ref faction_type) = faction_comp.faction_type {
            println!("  type:       {faction_type}");
        }
        if let Some(ref alignment) = faction_comp.alignment {
            println!("  alignment:  {alignment}");
        }
        if !faction_comp.values.is_empty() {
            println!("  values:     {}", faction_comp.values.join(", "));
        }
    }

    if let Some(event_comp) = &entity.components.event {
        if let Some(ref event_type) = event_comp.event_type {
            println!("  type:       {event_type}");
        }
        if let Some(ref date) = event_comp.date {
            println!("  date:       {date}");
        }
        if let Some(ref duration) = event_comp.duration {
            println!("  duration:   {duration}");
        }
        if let Some(ref outcome) = event_comp.outcome {
            println!("  outcome:    {outcome}");
        }
    }

    if let Some(item_comp) = &entity.components.item {
        if let Some(ref item_type) = item_comp.item_type {
            println!("  type:       {item_type}");
        }
        if let Some(ref rarity) = item_comp.rarity {
            println!("  rarity:     {rarity}");
        }
    }

    if let Some(lore_comp) = &entity.components.lore {
        if let Some(ref lore_type) = lore_comp.lore_type {
            println!("  type:       {lore_type}");
        }
        if let Some(ref source) = lore_comp.source {
            println!("  source:     {source}");
        }
    }

    // Generic properties
    if !entity.properties.is_empty() {
        let mut props: Vec<_> = entity.properties.iter().collect();
        props.sort_by_key(|(k, _)| (*k).clone());
        for (key, value) in props {
            println!("  {key}: {value}");
        }
    }

    // Tags
    if !entity.tags.is_empty() {
        println!();
        println!("  tags: {}", entity.tags.join(", "));
    }

    // Relationships
    if show_relationships {
        println!();
        let rels = world.relationships_of(entity.id);
        if rels.is_empty() {
            println!("  {} (none)", "Relationships:".dimmed());
        } else {
            println!("  {}", "Relationships:".dimmed());
            for rel in &rels {
                let other_id = if rel.source == entity.id {
                    rel.target
                } else {
                    rel.source
                };
                let other_name = world.entity_name(other_id);

                let phrase = if rel.source == entity.id {
                    // Outgoing: "self <phrase> target"
                    format_outgoing_rel(&rel.kind, other_name, rel.label.as_deref())
                } else {
                    // Incoming: "source <phrase> self"
                    format_incoming_rel(&rel.kind, other_name, rel.label.as_deref())
                };
                println!("    {phrase}");
            }
        }
    }

    Ok(())
}

fn format_outgoing_rel(
    kind: &ww_core::RelationshipKind,
    target: &str,
    label: Option<&str>,
) -> String {
    use ww_core::RelationshipKind::*;
    match kind {
        ContainedIn => format!("in {target}"),
        ConnectedTo => {
            if let Some(dir) = label {
                format!("{dir} to {target}")
            } else {
                format!("connected to {target}")
            }
        }
        LocatedAt => format!("located at {target}"),
        BasedAt => format!("based at {target}"),
        MemberOf => format!("member of {target}"),
        LeaderOf => format!("leads {target}"),
        AlliedWith => format!("allied with {target}"),
        RivalOf => format!("rival of {target}"),
        RelatedTo => format!("related to {target}"),
        OwnedBy => format!("owns {target}"),
        ParticipatedIn => format!("participated in {target}"),
        CausedBy => format!("caused by {target}"),
        References => format!("references {target}"),
        Custom(s) => format!("{s} {target}"),
    }
}

fn format_incoming_rel(
    kind: &ww_core::RelationshipKind,
    source: &str,
    label: Option<&str>,
) -> String {
    use ww_core::RelationshipKind::*;
    match kind {
        ContainedIn => format!("contains {source}"),
        ConnectedTo => {
            if let Some(dir) = label {
                format!("{dir} from {source}")
            } else {
                format!("connected from {source}")
            }
        }
        LocatedAt => format!("{source} is located here"),
        BasedAt => format!("{source} is based here"),
        MemberOf => format!("{source} is a member"),
        LeaderOf => format!("led by {source}"),
        AlliedWith => format!("allied with {source}"),
        RivalOf => format!("rival of {source}"),
        RelatedTo => format!("related to {source}"),
        OwnedBy => format!("owned by {source}"),
        ParticipatedIn => format!("{source} participated"),
        CausedBy => format!("caused {source}"),
        References => format!("referenced by {source}"),
        Custom(s) => format!("{s} from {source}"),
    }
}
