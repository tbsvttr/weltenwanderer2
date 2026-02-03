use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_ids
        .iter()
        .map(|id| {
            let entity = app.world.get_entity(*id);
            match entity {
                Some(e) => {
                    let kind_str = if let Some(subtype) = e.location_subtype() {
                        format!("{} ({})", e.kind, subtype)
                    } else {
                        e.kind.to_string()
                    };

                    let line = Line::from(vec![
                        Span::styled(&e.name, Style::default().fg(Color::White).bold()),
                        Span::raw("  "),
                        Span::styled(kind_str, Style::default().fg(Color::DarkGray)),
                    ]);
                    ListItem::new(line)
                }
                None => ListItem::new("???"),
            }
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!(" Entities ({}) ", app.filtered_ids.len())
    } else {
        format!(
            " Entities ({}) — filter: \"{}\" ",
            app.filtered_ids.len(),
            app.search_query
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .bold(),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.list_cursor));

    frame.render_stateful_widget(list, area, &mut state);
}
