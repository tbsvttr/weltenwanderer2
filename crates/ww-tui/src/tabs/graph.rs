//! Relationship graph tab.

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use ww_core::World;
use ww_core::entity::EntityId;

use super::{InputMode, Tab};

/// Graph tab state.
pub struct GraphTab {
    /// The world data.
    world: World,
    /// Scroll offset.
    scroll: u16,
}

impl GraphTab {
    /// Create a new graph tab for the given world.
    pub fn new(world: World) -> Self {
        Self { world, scroll: 0 }
    }
}

impl Tab for GraphTab {
    fn input_mode(&self) -> InputMode {
        InputMode::VimNav
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('g') => self.scroll = 0,
            _ => {}
        }
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_sub(1),
            MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_add(1),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut seen_pairs: HashSet<(EntityId, EntityId)> = HashSet::new();

        for rel in self.world.all_relationships() {
            let (a, b) = (rel.source.0, rel.target.0);
            let pair = if a < b {
                (rel.source, rel.target)
            } else {
                (rel.target, rel.source)
            };

            if !seen_pairs.insert(pair) && rel.bidirectional {
                continue;
            }

            let source_name = self.world.entity_name(rel.source);
            let target_name = self.world.entity_name(rel.target);

            let arrow = if rel.bidirectional {
                " <--> "
            } else {
                " ---> "
            };

            let label = if let Some(ref l) = rel.label {
                format!("{} ({})", rel.kind.as_phrase(), l)
            } else {
                rel.kind.as_phrase().to_string()
            };

            lines.push(Line::from(vec![
                Span::styled(format!("[{source_name}]"), Style::default().fg(Color::Cyan)),
                Span::styled(arrow.to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(label, Style::default().fg(Color::Yellow)),
                Span::styled(arrow.to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("[{target_name}]"),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "No relationships.",
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Summary
        lines.push(Line::from(""));
        let stats = self.world.entity_counts_by_kind();
        let mut sorted: Vec<_> = stats.iter().collect();
        sorted.sort_by_key(|(k, _)| format!("{k}"));
        let summary: Vec<String> = sorted.iter().map(|(k, v)| format!("{v} {k}")).collect();

        lines.push(Line::from(vec![Span::styled(
            format!(
                "{} entities, {} relationships",
                self.world.entity_count(),
                self.world.relationship_count()
            ),
            Style::default().fg(Color::DarkGray),
        )]));
        if !summary.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("({})", summary.join(", ")),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Relationship Graph ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        frame.render_widget(paragraph, area);
    }

    fn status_hint(&self) -> &str {
        "j/k:scroll  Tab:view  ?:help  q:quit"
    }
}
