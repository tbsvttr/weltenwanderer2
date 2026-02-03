use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let entity = match app.detail_entity_id.and_then(|id| app.world.get_entity(id)) {
        Some(e) => e,
        None => {
            let msg = Paragraph::new("No entity selected")
                .block(Block::default().title(" Detail ").borders(Borders::ALL));
            frame.render_widget(msg, area);
            return;
        }
    };

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Name and kind header
    let kind_str = if let Some(subtype) = entity.location_subtype() {
        format!("{} ({})", entity.kind, subtype)
    } else {
        entity.kind.to_string()
    };

    lines.push(Line::from(vec![
        Span::styled(entity.name.clone(), Style::default().fg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::styled(
            format!("[{kind_str}]"),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(""));

    // Description
    if !entity.description.is_empty() {
        for desc_line in entity.description.lines() {
            lines.push(Line::from(Span::styled(
                desc_line.trim().to_string(),
                Style::default().fg(Color::White),
            )));
        }
        lines.push(Line::from(""));
    }

    // Component-specific fields
    if let Some(char_comp) = &entity.components.character {
        lines.push(Line::from(Span::styled(
            "Character",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref species) = char_comp.species {
            lines.push(field_line("  species", species));
        }
        if let Some(ref occupation) = char_comp.occupation {
            lines.push(field_line("  occupation", occupation));
        }
        lines.push(field_line("  status", &format!("{:?}", char_comp.status)));
        if !char_comp.traits.is_empty() {
            lines.push(field_line("  traits", &char_comp.traits.join(", ")));
        }
        lines.push(Line::from(""));
    }

    if let Some(loc_comp) = &entity.components.location {
        lines.push(Line::from(Span::styled(
            "Location",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if loc_comp.location_type != "location" {
            lines.push(field_line("  type", &loc_comp.location_type));
        }
        if let Some(ref climate) = loc_comp.climate {
            lines.push(field_line("  climate", climate));
        }
        if let Some(ref terrain) = loc_comp.terrain {
            lines.push(field_line("  terrain", terrain));
        }
        if let Some(population) = loc_comp.population {
            lines.push(field_line("  population", &population.to_string()));
        }
        lines.push(Line::from(""));
    }

    if let Some(faction_comp) = &entity.components.faction {
        lines.push(Line::from(Span::styled(
            "Faction",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref ft) = faction_comp.faction_type {
            lines.push(field_line("  type", ft));
        }
        if !faction_comp.values.is_empty() {
            lines.push(field_line("  values", &faction_comp.values.join(", ")));
        }
        lines.push(Line::from(""));
    }

    if let Some(event_comp) = &entity.components.event {
        lines.push(Line::from(Span::styled(
            "Event",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref et) = event_comp.event_type {
            lines.push(field_line("  type", et));
        }
        if let Some(ref date) = event_comp.date {
            lines.push(field_line("  date", &date.to_string()));
        }
        if let Some(ref outcome) = event_comp.outcome {
            lines.push(field_line("  outcome", outcome));
        }
        lines.push(Line::from(""));
    }

    if let Some(item_comp) = &entity.components.item {
        lines.push(Line::from(Span::styled(
            "Item",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref it) = item_comp.item_type {
            lines.push(field_line("  type", it));
        }
        if let Some(ref rarity) = item_comp.rarity {
            lines.push(field_line("  rarity", rarity));
        }
        lines.push(Line::from(""));
    }

    if let Some(lore_comp) = &entity.components.lore {
        lines.push(Line::from(Span::styled(
            "Lore",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref lt) = lore_comp.lore_type {
            lines.push(field_line("  type", lt));
        }
        if let Some(ref source) = lore_comp.source {
            lines.push(field_line("  source", source));
        }
        lines.push(Line::from(""));
    }

    // Properties
    if !entity.properties.is_empty() {
        lines.push(Line::from(Span::styled(
            "Properties",
            Style::default().fg(Color::Yellow).bold(),
        )));
        let mut props: Vec<_> = entity.properties.iter().collect();
        props.sort_by_key(|(k, _)| (*k).clone());
        for (key, value) in props {
            lines.push(field_line(&format!("  {key}"), &value.to_string()));
        }
        lines.push(Line::from(""));
    }

    // Relationships
    let rels = app.world.relationships_of(entity.id);
    if !rels.is_empty() {
        lines.push(Line::from(Span::styled(
            "Relationships",
            Style::default().fg(Color::Yellow).bold(),
        )));
        for rel in &rels {
            let other_id = if rel.source == entity.id {
                rel.target
            } else {
                rel.source
            };
            let other_name = app
                .world
                .get_entity(other_id)
                .map(|e| e.name.as_str())
                .unwrap_or("???");

            let phrase = rel.kind.as_phrase();
            let label = if let Some(ref l) = rel.label {
                format!("{phrase} ({l}) -> {other_name}")
            } else {
                format!("{phrase} -> {other_name}")
            };

            lines.push(Line::from(vec![
                Span::styled("  ".to_string(), Style::default()),
                Span::styled(label, Style::default().fg(Color::Green)),
            ]));
        }
    }

    let title = format!(" {} ", entity.name);
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn field_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<14}"), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}
