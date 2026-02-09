//! Interactive fiction play session screen.

use macroquad::prelude::*;

use ww_fiction::FictionSession;

use crate::app::AppState;
use crate::theme::font::draw_pixel_text;
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W, mouse_canvas_position};
use crate::widget::Rect2;
use crate::widget::panel::draw_panel;
use crate::widget::text_area;

use super::{Screen, Transition};

/// Play session screen state.
pub struct PlayScreen {
    /// The fiction session.
    session: Option<FictionSession>,
    /// Text output log.
    output_log: String,
    /// Current input text.
    input_text: String,
    /// Scroll offset for the output area.
    scroll: usize,
    /// Initialization error.
    error: Option<String>,
}

impl Default for PlayScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayScreen {
    /// Create a new play screen.
    pub fn new() -> Self {
        Self {
            session: None,
            output_log: String::new(),
            input_text: String::new(),
            scroll: 0,
            error: None,
        }
    }

    fn initialize(&mut self, app: &AppState) {
        let Some(world) = &app.world else {
            self.error = Some("No world loaded".to_string());
            return;
        };

        match FictionSession::new(world.clone()) {
            Ok(mut session) => {
                // Initial look
                match session.process("look") {
                    Ok(output) => {
                        self.output_log = output;
                    }
                    Err(e) => {
                        self.output_log = format!("Error: {e}");
                    }
                }
                self.session = Some(session);
            }
            Err(e) => {
                self.error = Some(format!("{e}"));
            }
        }
    }
}

impl Screen for PlayScreen {
    fn update(&mut self, app: &mut AppState) -> Transition {
        if self.session.is_none() && self.error.is_none() {
            self.initialize(app);
        }

        if is_key_pressed(KeyCode::Escape) {
            return Transition::Pop;
        }

        // Mouse: tab click
        let (mx, my) = mouse_canvas_position();
        if let Some(t) = super::handle_tab_click(mx, my, 3) {
            return t;
        }

        // Text input
        for ch in crate::input::typed_chars() {
            self.input_text.push(ch);
        }
        if crate::input::backspace_pressed() {
            self.input_text.pop();
        }

        // Submit command
        if crate::input::enter_pressed()
            && !self.input_text.is_empty()
            && let Some(session) = &mut self.session
        {
            let input = self.input_text.clone();
            self.input_text.clear();

            self.output_log.push_str(&format!("\n\n> {input}\n\n"));

            match session.process(&input) {
                Ok(output) => {
                    self.output_log.push_str(&output);
                }
                Err(e) => {
                    self.output_log.push_str(&format!("Error: {e}"));
                }
            }

            // Auto-scroll to bottom
            self.scroll = usize::MAX;
        }

        // Scroll (keyboard + mouse wheel)
        if is_key_pressed(KeyCode::PageUp) {
            self.scroll = self.scroll.saturating_sub(5);
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.scroll = self.scroll.saturating_add(5);
        }
        let wheel = crate::input::scroll_y();
        if wheel > 0.0 {
            self.scroll = self.scroll.saturating_sub(3);
        } else if wheel < 0.0 {
            self.scroll = self.scroll.saturating_add(3);
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = mouse_canvas_position();

        // Tab bar
        let tab_area = Rect2::new(0.0, 0.0, CANVAS_W, 14.0);
        crate::widget::tabs::draw_tabs(&app.font, super::TAB_LABELS, 3, &tab_area, mx, my);

        // Output panel (below tab bar)
        let output_area = Rect2::new(4.0, 18.0, CANVAS_W - 8.0, CANVAS_H - 40.0);
        draw_panel(&output_area);
        let output_inner = output_area.inset(3.0);

        if let Some(ref err) = self.error {
            draw_pixel_text(&app.font, err, output_inner.x, output_inner.y, palette::RED);
        } else {
            let total_lines =
                text_area::draw_text_area(&app.font, &self.output_log, self.scroll, &output_inner);
            // Auto-scroll: if scroll is MAX, set to last page
            if self.scroll == usize::MAX {
                let visible = (output_inner.h / 10.0) as usize;
                let _ = text_area::draw_text_area(
                    &app.font,
                    &self.output_log,
                    total_lines.saturating_sub(visible),
                    &output_inner,
                );
            }
        }

        // Input area
        let input_area = Rect2::new(4.0, CANVAS_H - 18.0, CANVAS_W - 8.0, 14.0);
        crate::widget::input::draw_input(
            &app.font,
            &self.input_text,
            "type a command...",
            true,
            &input_area,
        );

        // Status
        draw_pixel_text(
            &app.font,
            "Enter: send | PgUp/PgDn: scroll | Esc: back",
            2.0,
            CANVAS_H - 2.0,
            palette::DARK_GRAY,
        );
    }
}
