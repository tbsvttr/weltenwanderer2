//! Input line with cursor and autocomplete popup.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

use crate::tabs::solo::SoloTab;

/// Draw the input line and status bar.
pub fn draw(frame: &mut Frame, tab: &SoloTab, input_area: Rect, status_area: Rect) {
    // Input field
    let display_text = format!("> {}", tab.input_text);
    let input = Paragraph::new(display_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(input, input_area);

    // Place cursor: offset by 2 for "> " prefix, plus 1 for left border
    let cursor_x = input_area.x + 1 + 2 + tab.input_cursor as u16;
    let cursor_y = input_area.y + 1;
    if cursor_x < input_area.x + input_area.width - 1 {
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }

    // Autocomplete popup (drawn above the input)
    if tab.completion.active && !tab.completion.candidates.is_empty() {
        draw_completion_popup(frame, tab, input_area);
    }

    // Status bar -- context-aware hints
    let status_spans = if tab.completion.active {
        vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(":cycle  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(":accept  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(":cancel  "),
        ]
    } else {
        vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(":complete  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(":send  "),
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Yellow)),
            Span::raw(":scroll  "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(":help  "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
            Span::raw(":quit"),
        ]
    };
    let status = Paragraph::new(Line::from(status_spans))
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black));

    frame.render_widget(status, status_area);
}

/// Draw the autocomplete popup above the input area.
fn draw_completion_popup(frame: &mut Frame, tab: &SoloTab, input_area: Rect) {
    let max_visible = 6_u16;
    let count = tab.completion.candidates.len() as u16;
    let popup_height = count.min(max_visible) + 2; // +2 for borders
    let popup_width = tab
        .completion
        .candidates
        .iter()
        .map(|c| c.len() as u16)
        .max()
        .unwrap_or(10)
        .min(input_area.width.saturating_sub(4))
        + 4; // padding

    let popup_y = input_area.y.saturating_sub(popup_height);
    let popup_area = Rect::new(input_area.x + 1, popup_y, popup_width, popup_height);

    let items: Vec<ListItem> = tab
        .completion
        .candidates
        .iter()
        .map(|c| ListItem::new(Span::raw(format!(" {c}"))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(tab.completion.index));

    frame.render_widget(Clear, popup_area);
    frame.render_stateful_widget(list, popup_area, &mut state);
}
