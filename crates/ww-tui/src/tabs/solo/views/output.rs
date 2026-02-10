//! Scrollable output panel for session log.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::shared::OutputStyle;
use crate::tabs::solo::SoloTab;

/// Draw the scrollable output panel.
pub fn draw(frame: &mut Frame, tab: &SoloTab, area: Rect) {
    let lines: Vec<Line> = tab
        .output_lines
        .iter()
        .map(|ol| {
            let (prefix, color, modifier) = match ol.style {
                OutputStyle::Command => ("> ", Color::Yellow, Modifier::BOLD),
                OutputStyle::Result => ("", Color::White, Modifier::empty()),
                OutputStyle::Error => ("", Color::Red, Modifier::empty()),
                OutputStyle::System => ("", Color::Cyan, Modifier::ITALIC),
            };
            Line::from(Span::styled(
                format!("{prefix}{}", ol.text),
                Style::default().fg(color).add_modifier(modifier),
            ))
        })
        .collect();

    // Calculate scroll: show the bottom by default.
    let inner_width = area.width.saturating_sub(2) as usize;
    let total_wrapped: u16 = lines
        .iter()
        .map(|l| {
            let len = l.width();
            if inner_width == 0 {
                1
            } else {
                len.max(1).div_ceil(inner_width) as u16
            }
        })
        .sum();

    let visible_height = area.height.saturating_sub(2);
    let max_scroll = total_wrapped.saturating_sub(visible_height);
    let scroll = max_scroll.saturating_sub(tab.output_scroll);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Session ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}
