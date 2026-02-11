//! Top action bar with clickable buttons.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// An action button definition.
struct ActionButton {
    /// Display label.
    label: &'static str,
    /// Command to execute or prefill. Trailing space means prefill.
    command: &'static str,
    /// Whether this button requires the chaos/scene system.
    requires_chaos: bool,
}

/// All action buttons in display order.
const BUTTONS: &[&[ActionButton]] = &[
    // Row 1
    &[
        ActionButton {
            label: "Oracle",
            command: "ask ",
            requires_chaos: false,
        },
        ActionButton {
            label: "Event",
            command: "event",
            requires_chaos: false,
        },
        ActionButton {
            label: "Scene",
            command: "scene ",
            requires_chaos: true,
        },
        ActionButton {
            label: "EndScn",
            command: "end scene ",
            requires_chaos: true,
        },
        ActionButton {
            label: "Check",
            command: "check ",
            requires_chaos: false,
        },
        ActionButton {
            label: "Roll",
            command: "roll ",
            requires_chaos: false,
        },
    ],
    // Row 2
    &[
        ActionButton {
            label: "Sheet",
            command: "sheet",
            requires_chaos: false,
        },
        ActionButton {
            label: "Panic",
            command: "panic",
            requires_chaos: false,
        },
        ActionButton {
            label: "Encntr",
            command: "encounter ",
            requires_chaos: false,
        },
        ActionButton {
            label: "Note",
            command: "note ",
            requires_chaos: false,
        },
        ActionButton {
            label: "Jrnl",
            command: "journal",
            requires_chaos: false,
        },
        ActionButton {
            label: "Help",
            command: "help",
            requires_chaos: false,
        },
    ],
];

/// Draw the action bar. Two rows of styled button labels.
pub fn draw(frame: &mut Frame, area: Rect, enable_chaos: bool) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    for (row_idx, row_buttons) in BUTTONS.iter().enumerate() {
        if row_idx >= rows.len() {
            break;
        }

        let spans: Vec<Span> = row_buttons
            .iter()
            .filter(|btn| !btn.requires_chaos || enable_chaos)
            .flat_map(|btn| {
                let is_prefill = btn.command.ends_with(' ');
                let style = if is_prefill {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                };
                vec![
                    Span::styled(format!(" {} ", btn.label), style),
                    Span::raw(" "),
                ]
            })
            .collect();

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(line, rows[row_idx]);
    }
}

/// Check if a mouse click at (col, row) hits a button. Returns the command string.
pub fn hit_test(col: u16, row: u16, area: Rect, enable_chaos: bool) -> Option<&'static str> {
    if row < area.y || row >= area.y + area.height || col < area.x {
        return None;
    }

    let local_row = (row - area.y) as usize;
    if local_row >= BUTTONS.len() {
        return None;
    }

    let buttons = BUTTONS[local_row];

    // Each button: " label " + " " separator = label.len() + 3
    let mut x = area.x;
    for btn in buttons {
        if btn.requires_chaos && !enable_chaos {
            continue;
        }
        let btn_width = btn.label.len() as u16 + 2; // " label "
        if col >= x && col < x + btn_width {
            return Some(btn.command);
        }
        x += btn_width + 1; // +1 for separator space
    }

    None
}
