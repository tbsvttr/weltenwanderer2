//! Shared utilities for TUI views: layout helpers, output types, and popups.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Visual style for an output line (used by play and solo tabs).
#[derive(Debug, Clone, Copy)]
pub enum OutputStyle {
    /// A command the user entered (yellow, "> " prefix).
    Command,
    /// Normal output from the session (white).
    Result,
    /// Error output (red).
    Error,
    /// System message like intro or help (cyan).
    System,
}

/// A single line of output in a session log.
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// Visual style of this line.
    pub style: OutputStyle,
    /// The text content.
    pub text: String,
}

/// Create a centered rectangle as a percentage of the given area.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Draw a global help popup overlay.
pub fn draw_help_popup(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());

    let help_text = vec![
        Line::from("Keyboard Shortcuts").style(Style::default().bold()),
        Line::from(""),
        Line::from("Navigation Tabs:"),
        Line::from("  1-7 / Tab   Switch tab (VimNav mode)"),
        Line::from("  Ctrl+1..7   Switch tab (TextInput mode)"),
        Line::from(""),
        Line::from("Explorer / Graph / Timeline / Sheet / Dice:"),
        Line::from("  j / k       Move down / up"),
        Line::from("  g / G       Go to top / bottom"),
        Line::from("  Enter       Select / drill in"),
        Line::from("  Esc         Go back"),
        Line::from("  /           Search (explorer only)"),
        Line::from("  q           Quit"),
        Line::from(""),
        Line::from("Play / Solo:"),
        Line::from("  Enter       Submit command"),
        Line::from("  Tab         Autocomplete"),
        Line::from("  Esc         Clear input"),
        Line::from("  Arrow keys  Scroll / move cursor"),
        Line::from(""),
        Line::from("  ?           Toggle this help"),
        Line::from("  Ctrl+C      Quit"),
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
