//! Interactive fiction play tab.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use ww_core::World;
use ww_fiction::FictionSession;

use crate::shared::{OutputLine, OutputStyle};
use crate::tabs::{InputMode, Tab};

/// Interactive fiction play tab state.
pub struct PlayTab {
    /// The fiction session (lazily initialized).
    session: Option<FictionSession>,
    /// Styled output log.
    output_lines: Vec<OutputLine>,
    /// Scroll offset from the bottom.
    output_scroll: u16,
    /// Current input text.
    input_text: String,
    /// Cursor position within input text.
    input_cursor: usize,
    /// Initialization error.
    error: Option<String>,
}

impl PlayTab {
    /// Create a new play tab for the given world.
    pub fn new(world: World) -> Self {
        let mut tab = Self {
            session: None,
            output_lines: Vec::new(),
            output_scroll: 0,
            input_text: String::new(),
            input_cursor: 0,
            error: None,
        };
        tab.initialize(world);
        tab
    }

    fn initialize(&mut self, world: World) {
        match FictionSession::new(world) {
            Ok(mut session) => {
                self.push_output(
                    OutputStyle::System,
                    "Interactive Fiction\n\
                     Explore the world, talk to characters,\n\
                     pick up items. Type 'help' for commands.",
                );
                match session.process("look") {
                    Ok(output) => self.push_output(OutputStyle::Result, &output),
                    Err(e) => self.push_output(OutputStyle::Error, &format!("Error: {e}")),
                }
                self.session = Some(session);
            }
            Err(e) => {
                self.error = Some(format!("{e}"));
            }
        }
    }

    fn submit_input(&mut self) {
        let input = self.input_text.trim().to_string();
        if input.is_empty() {
            return;
        }
        self.input_text.clear();
        self.input_cursor = 0;

        self.push_output(OutputStyle::Command, &input);

        if let Some(session) = &mut self.session {
            match session.process(&input) {
                Ok(output) => {
                    if !output.is_empty() {
                        self.push_output(OutputStyle::Result, &output);
                    }
                }
                Err(e) => {
                    self.push_output(OutputStyle::Error, &e.to_string());
                }
            }
        }
    }

    fn push_output(&mut self, style: OutputStyle, text: &str) {
        for line in text.lines() {
            self.output_lines.push(OutputLine {
                style,
                text: line.to_string(),
            });
        }
        self.output_scroll = 0;
    }
}

impl Tab for PlayTab {
    fn input_mode(&self) -> InputMode {
        InputMode::TextInput
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => self.submit_input(),
            KeyCode::Esc => {
                if !self.input_text.is_empty() {
                    self.input_text.clear();
                    self.input_cursor = 0;
                }
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    let prev = self.input_text[..self.input_cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_text.remove(prev);
                    self.input_cursor = prev;
                }
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    let prev = self.input_text[..self.input_cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.input_cursor = prev;
                }
            }
            KeyCode::Right => {
                if self.input_cursor < self.input_text.len() {
                    let next = self.input_text[self.input_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.input_cursor + i)
                        .unwrap_or(self.input_text.len());
                    self.input_cursor = next;
                }
            }
            KeyCode::Home => self.input_cursor = 0,
            KeyCode::End => self.input_cursor = self.input_text.len(),
            KeyCode::Up => self.output_scroll = self.output_scroll.saturating_add(1),
            KeyCode::Down => self.output_scroll = self.output_scroll.saturating_sub(1),
            KeyCode::Char(c) => {
                self.input_text.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
            }
            _ => {}
        }
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.output_scroll = self.output_scroll.saturating_add(1);
            }
            MouseEventKind::ScrollDown => {
                self.output_scroll = self.output_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Output
                Constraint::Length(3), // Input
            ])
            .split(area);

        // Output panel
        if let Some(ref err) = self.error {
            let msg = Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)))
                .block(
                    Block::default()
                        .title(" Play ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                );
            frame.render_widget(msg, chunks[0]);
        } else {
            let lines: Vec<Line> = self
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

            let inner_width = chunks[0].width.saturating_sub(2) as usize;
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

            let visible_height = chunks[0].height.saturating_sub(2);
            let max_scroll = total_wrapped.saturating_sub(visible_height);
            let scroll = max_scroll.saturating_sub(self.output_scroll);

            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" Play ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0));

            frame.render_widget(paragraph, chunks[0]);
        }

        // Input field
        let display_text = format!("> {}", self.input_text);
        let input = Paragraph::new(display_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(input, chunks[1]);

        // Cursor
        let cursor_x = chunks[1].x + 1 + 2 + self.input_cursor as u16;
        let cursor_y = chunks[1].y + 1;
        if cursor_x < chunks[1].x + chunks[1].width - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    fn status_hint(&self) -> &str {
        "Enter:send  Esc:clear  \u{2191}\u{2193}:scroll  Ctrl+C:quit"
    }
}
