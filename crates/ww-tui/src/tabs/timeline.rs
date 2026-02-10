//! Chronological timeline tab.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use ww_core::World;
use ww_core::timeline::Timeline;

use super::{InputMode, Tab};

/// Timeline tab state.
pub struct TimelineTab {
    /// The world data.
    world: World,
    /// Cursor position in the event list.
    cursor: usize,
}

impl TimelineTab {
    /// Create a new timeline tab for the given world.
    pub fn new(world: World) -> Self {
        Self { world, cursor: 0 }
    }

    fn entry_count(&self) -> usize {
        Timeline::from_world(&self.world).len()
    }
}

impl Tab for TimelineTab {
    fn input_mode(&self) -> InputMode {
        InputMode::VimNav
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let count = self.entry_count();
                if self.cursor + 1 < count {
                    self.cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Char('g') => self.cursor = 0,
            KeyCode::Char('G') => {
                let count = self.entry_count();
                if count > 0 {
                    self.cursor = count - 1;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.cursor = self.cursor.saturating_sub(1),
            MouseEventKind::ScrollDown => {
                let count = self.entry_count();
                if self.cursor + 1 < count {
                    self.cursor += 1;
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let timeline = Timeline::from_world(&self.world);
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
            .highlight_symbol("\u{25b6} ");

        let mut state = ListState::default();
        state.select(Some(self.cursor));

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn status_hint(&self) -> &str {
        "j/k:navigate  Tab:view  ?:help  q:quit"
    }
}
