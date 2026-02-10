//! Solo TTRPG session tab with action buttons, tab completion, and sidebar.

pub mod views;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::*;

use ww_core::World;
use ww_solo::{SoloConfig, SoloSession};

use crate::shared::{OutputLine, OutputStyle};
use crate::tabs::{InputMode, Tab};

/// Tab-completion state.
#[derive(Debug, Default)]
pub struct CompletionState {
    /// Candidate strings.
    pub candidates: Vec<String>,
    /// Currently selected candidate index.
    pub index: usize,
    /// Whether the popup is visible.
    pub active: bool,
    /// The input text before completion started.
    pub original_input: String,
}

/// Solo TTRPG tab state.
pub struct SoloTab {
    /// The solo session engine.
    pub session: SoloSession,
    /// Styled output log.
    pub output_lines: Vec<OutputLine>,
    /// Scroll offset from the bottom (0 = fully scrolled down).
    pub output_scroll: u16,
    /// Current input text.
    pub input_text: String,
    /// Cursor position within input text (byte offset).
    pub input_cursor: usize,
    /// Tab-completion state.
    pub completion: CompletionState,
    /// Whether the help popup is visible.
    pub show_help: bool,
}

impl SoloTab {
    /// Create a new solo tab from a world and config.
    pub fn new(world: World, config: SoloConfig) -> Result<Self, String> {
        let session =
            SoloSession::new(world, config).map_err(|e| format!("failed to start session: {e}"))?;
        let intro = session.intro();
        let mut tab = Self {
            session,
            output_lines: Vec::new(),
            output_scroll: 0,
            input_text: String::new(),
            input_cursor: 0,
            completion: CompletionState::default(),
            show_help: false,
        };
        tab.push_output(OutputStyle::System, &intro);
        Ok(tab)
    }

    /// Submit the current input text as a command.
    pub fn submit_input(&mut self) {
        let input = self.input_text.trim().to_string();
        if input.is_empty() {
            return;
        }
        self.clear_completion();
        self.execute_command(&input);
        self.input_text.clear();
        self.input_cursor = 0;
    }

    /// Execute a command string (from button or input).
    pub fn execute_command(&mut self, cmd: &str) {
        self.push_output(OutputStyle::Command, cmd);

        match self.session.process(cmd) {
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

    /// Prefill the input with a command prefix (from button click).
    pub fn prefill_input(&mut self, text: &str) {
        self.input_text = text.to_string();
        self.input_cursor = self.input_text.len();
        self.clear_completion();
    }

    /// Trigger or cycle tab completion.
    pub fn tab_complete(&mut self) {
        if self.completion.active {
            self.cycle_completion_forward();
        } else {
            self.start_completion();
        }
    }

    /// Accept the current completion selection.
    pub fn accept_completion(&mut self) {
        if !self.completion.active {
            return;
        }
        let expects_more = self.input_text.ends_with(' ');
        self.clear_completion();
        if expects_more {
            self.start_completion();
        }
    }

    /// Start a new completion from the current input text.
    fn start_completion(&mut self) {
        let candidates = self.session.completions(&self.input_text);
        if candidates.len() == 1 {
            self.input_text = candidates[0].clone();
            self.input_cursor = self.input_text.len();
            if self.input_text.ends_with(' ') {
                let sub = self.session.completions(&self.input_text);
                if !sub.is_empty() {
                    self.completion.original_input = self.input_text.clone();
                    self.completion.candidates = sub;
                    self.completion.index = 0;
                    self.completion.active = true;
                    self.input_text = self.completion.candidates[0].clone();
                    self.input_cursor = self.input_text.len();
                }
            }
        } else if !candidates.is_empty() {
            self.completion.original_input = self.input_text.clone();
            self.completion.candidates = candidates;
            self.completion.index = 0;
            self.completion.active = true;
            self.input_text = self.completion.candidates[0].clone();
            self.input_cursor = self.input_text.len();
        }
    }

    /// Cycle forward through completion candidates.
    fn cycle_completion_forward(&mut self) {
        if !self.completion.candidates.is_empty() {
            self.completion.index = (self.completion.index + 1) % self.completion.candidates.len();
            self.input_text = self.completion.candidates[self.completion.index].clone();
            self.input_cursor = self.input_text.len();
        }
    }

    /// Shift-tab: cycle backwards through completions.
    pub fn tab_complete_prev(&mut self) {
        if self.completion.active && !self.completion.candidates.is_empty() {
            if self.completion.index == 0 {
                self.completion.index = self.completion.candidates.len() - 1;
            } else {
                self.completion.index -= 1;
            }
            self.input_text = self.completion.candidates[self.completion.index].clone();
            self.input_cursor = self.input_text.len();
        }
    }

    /// Clear the completion state.
    pub fn clear_completion(&mut self) {
        self.completion.active = false;
        self.completion.candidates.clear();
        self.completion.index = 0;
        self.completion.original_input.clear();
    }

    /// Push a character to the input at the cursor position.
    pub fn push_char(&mut self, c: char) {
        self.clear_completion();
        self.input_text.insert(self.input_cursor, c);
        self.input_cursor += c.len_utf8();
    }

    /// Delete the character before the cursor.
    pub fn backspace(&mut self) {
        self.clear_completion();
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

    /// Delete the character at the cursor.
    pub fn delete_char(&mut self) {
        self.clear_completion();
        if self.input_cursor < self.input_text.len() {
            self.input_text.remove(self.input_cursor);
        }
    }

    /// Move cursor left.
    pub fn cursor_left(&mut self) {
        if self.input_cursor > 0 {
            let prev = self.input_text[..self.input_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input_cursor = prev;
        }
    }

    /// Move cursor right.
    pub fn cursor_right(&mut self) {
        if self.input_cursor < self.input_text.len() {
            let next = self.input_text[self.input_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.input_cursor + i)
                .unwrap_or(self.input_text.len());
            self.input_cursor = next;
        }
    }

    /// Move cursor to start of input.
    pub fn cursor_home(&mut self) {
        self.input_cursor = 0;
    }

    /// Move cursor to end of input.
    pub fn cursor_end(&mut self) {
        self.input_cursor = self.input_text.len();
    }

    /// Scroll output up.
    pub fn scroll_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_add(1);
    }

    /// Scroll output down.
    pub fn scroll_down(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(1);
    }

    /// Append styled text to the output log.
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

impl Tab for SoloTab {
    fn input_mode(&self) -> InputMode {
        InputMode::TextInput
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                if self.completion.active {
                    self.accept_completion();
                } else {
                    self.submit_input();
                }
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.tab_complete_prev();
                } else {
                    self.tab_complete();
                }
            }
            KeyCode::BackTab => self.tab_complete_prev(),
            KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                } else if self.completion.active {
                    self.input_text = self.completion.original_input.clone();
                    self.input_cursor = self.input_text.len();
                    self.clear_completion();
                } else if !self.input_text.is_empty() {
                    self.input_text.clear();
                    self.input_cursor = 0;
                }
            }
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete_char(),
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            KeyCode::Home => self.cursor_home(),
            KeyCode::End => self.cursor_end(),
            KeyCode::Up => {
                if self.completion.active {
                    self.tab_complete_prev();
                } else {
                    self.scroll_up();
                }
            }
            KeyCode::Down => {
                if self.completion.active {
                    self.tab_complete();
                } else {
                    self.scroll_down();
                }
            }
            KeyCode::Char('?') if self.input_text.is_empty() && !self.completion.active => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char(c) => self.push_char(c),
            _ => {}
        }
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Action bar hit testing (approximate: action bar at rows 1-2 in tab content area)
                let action_area = Rect::new(0, 0, 80, 2);
                if let Some(cmd) =
                    views::actions::hit_test(mouse.column, mouse.row.saturating_sub(1), action_area)
                {
                    if cmd.ends_with(' ') {
                        self.prefill_input(cmd);
                    } else {
                        self.execute_command(cmd);
                    }
                }
            }
            MouseEventKind::ScrollUp => self.scroll_up(),
            MouseEventKind::ScrollDown => self.scroll_down(),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Action bar
                Constraint::Min(5),    // Main content
                Constraint::Length(3), // Input
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Action bar
        views::actions::draw(frame, chunks[0]);

        // Main content: output (65%) + sidebar (35%)
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(chunks[1]);

        views::output::draw(frame, self, content[0]);
        views::sidebar::draw(frame, self, content[1]);

        // Input + status
        views::input::draw(frame, self, chunks[2], chunks[3]);

        // Help popup overlay
        if self.show_help {
            views::draw_help_popup(frame);
        }
    }

    fn status_hint(&self) -> &str {
        if self.completion.active {
            "Tab:cycle  Enter:accept  Esc:cancel"
        } else {
            "Tab:complete  Enter:send  \u{2191}\u{2193}:scroll  ?:help  Ctrl+C:quit"
        }
    }
}
