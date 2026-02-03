use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use ww_core::timeline::Timeline;

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let timeline = Timeline::from_world(&app.world);
    let entries = timeline.entries();

    if entries.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("  No events with dates found.").block(
            Block::default()
                .title(" Timeline ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| {
            let date_str = entry.date.to_string();
            let event_type = entry
                .entity
                .components
                .event
                .as_ref()
                .and_then(|e| e.event_type.as_deref())
                .unwrap_or("");

            let type_tag = if event_type.is_empty() {
                String::new()
            } else {
                format!(" [{}]", event_type)
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{date_str:>30}"),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(&entry.entity.name, Style::default().fg(Color::White).bold()),
                Span::styled(type_tag, Style::default().fg(Color::Yellow)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Timeline ({} events) ", entries.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold())
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.timeline_cursor));

    frame.render_stateful_widget(list, area, &mut state);
}
