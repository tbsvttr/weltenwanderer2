//! Title screen with world directory picker.

use macroquad::prelude::*;

use crate::app::AppState;
use crate::theme::font::{draw_pixel_text, measure_text_width};
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W, mouse_canvas_position};
use crate::widget::Rect2;
use crate::widget::button::draw_button;
use crate::widget::input::draw_input;

use super::{Screen, ScreenId, Transition};

/// Title screen state.
pub struct TitleScreen {
    /// Path typed into the input field.
    pub dir_input: String,
    /// Error message to display.
    pub error: Option<String>,
    /// Whether the input field is active.
    pub input_active: bool,
}

impl Default for TitleScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl TitleScreen {
    /// Create a new title screen.
    pub fn new() -> Self {
        Self {
            dir_input: String::new(),
            error: None,
            input_active: true,
        }
    }

    /// Create with a preset directory path.
    pub fn with_dir(dir: &str) -> Self {
        Self {
            dir_input: dir.to_string(),
            error: None,
            input_active: true,
        }
    }
}

impl Screen for TitleScreen {
    fn update(&mut self, app: &mut AppState) -> Transition {
        if is_key_pressed(KeyCode::Escape) {
            app.should_quit = true;
            return Transition::None;
        }

        // Text input for directory path
        for ch in crate::input::typed_chars() {
            self.dir_input.push(ch);
        }
        if crate::input::backspace_pressed() {
            self.dir_input.pop();
        }

        // Open button click
        let (mx, my) = crate::theme::mouse_canvas_position();
        let btn_w = 80.0;
        let btn_x = (CANVAS_W - btn_w) / 2.0;
        let btn_area = Rect2::new(btn_x, 148.0, btn_w, 16.0);
        let btn_clicked = btn_area.contains(mx, my) && is_mouse_button_pressed(MouseButton::Left);

        // Enter or button click to load world
        if (crate::input::enter_pressed() || btn_clicked) && !self.dir_input.is_empty() {
            let path = std::path::Path::new(&self.dir_input);
            match app.load_world(path) {
                Ok(()) => {
                    self.error = None;
                    return Transition::Replace(ScreenId::Explorer);
                }
                Err(e) => {
                    self.error = Some(e);
                }
            }
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = mouse_canvas_position();

        // Title
        let title = "WELTENWANDERER";
        let title_w = measure_text_width(title);
        let title_x = (CANVAS_W - title_w) / 2.0;
        draw_pixel_text(&app.font, title, title_x, 60.0, palette::YELLOW);

        // Subtitle
        let sub = "a creative engine for worldbuilding";
        let sub_w = measure_text_width(sub);
        let sub_x = (CANVAS_W - sub_w) / 2.0;
        draw_pixel_text(&app.font, sub, sub_x, 75.0, palette::LIGHT_GRAY);

        // Decorative line
        let line_w = 200.0;
        let line_x = (CANVAS_W - line_w) / 2.0;
        draw_rectangle(line_x, 90.0, line_w, 1.0, palette::DARK_GRAY);

        // World directory input
        let prompt = "World directory:";
        let prompt_w = measure_text_width(prompt);
        let prompt_x = (CANVAS_W - prompt_w) / 2.0;
        draw_pixel_text(&app.font, prompt, prompt_x, 110.0, palette::LIGHT_GRAY);

        let input_w = 300.0;
        let input_x = (CANVAS_W - input_w) / 2.0;
        let input_area = Rect2::new(input_x, 124.0, input_w, 14.0);
        // We don't mutate here (draw only), input is handled in update via typed_chars
        draw_input(
            &app.font,
            &self.dir_input,
            "path/to/world",
            self.input_active,
            &input_area,
        );

        // Open button
        let btn_w = 80.0;
        let btn_x = (CANVAS_W - btn_w) / 2.0;
        let btn_area = Rect2::new(btn_x, 148.0, btn_w, 16.0);
        draw_button(&app.font, "OPEN", &btn_area, mx, my);

        // Error message
        if let Some(ref err) = self.error {
            let err_w = measure_text_width(err);
            let err_x = ((CANVAS_W - err_w) / 2.0).max(4.0);
            draw_pixel_text(&app.font, err, err_x, 174.0, palette::RED);
        }

        // Help text
        let help = "Enter: open  |  Esc: quit";
        let help_w = measure_text_width(help);
        let help_x = (CANVAS_W - help_w) / 2.0;
        draw_pixel_text(&app.font, help, help_x, CANVAS_H - 20.0, palette::DARK_GRAY);
    }
}
