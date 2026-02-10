//! View rendering for the solo tab.

pub mod actions;
pub mod input;
pub mod output;
pub mod sidebar;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::shared::centered_rect;

/// Draw a help popup overlay for the solo tab.
pub fn draw_help_popup(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());

    let help_text = vec![
        Line::from("Solo Session Controls").style(Style::default().bold()),
        Line::from(""),
        Line::from("  Enter       Submit command"),
        Line::from("  Tab         Autocomplete (cycle forward)"),
        Line::from("  Shift+Tab   Cycle backward"),
        Line::from("  Esc         Clear completion / input"),
        Line::from("  \u{2191} / \u{2193}       Scroll output"),
        Line::from("  \u{2190} / \u{2192}       Move cursor in input"),
        Line::from("  Home / End  Jump to start / end"),
        Line::from("  Ctrl+C      Quit"),
        Line::from(""),
        Line::from("  Click action buttons to execute or prefill."),
        Line::from("  Click threads/NPCs for quick actions."),
        Line::from("  Scroll wheel on output to scroll."),
    ];

    let popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}
