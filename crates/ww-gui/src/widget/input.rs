//! Single-line text input field with blinking cursor.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::font::{PixelFont, draw_pixel_text};
use crate::theme::palette;

/// Draw a text input field with a blinking cursor.
///
/// `value` is the current text. `active` determines if the field accepts input.
/// Returns the updated text value.
pub fn draw_input(
    font: &PixelFont,
    value: &str,
    placeholder: &str,
    active: bool,
    area: &Rect2,
) -> String {
    // Background
    let fill = if active {
        palette::BLACK
    } else {
        palette::DARK_BLUE
    };
    let border = if active {
        palette::YELLOW
    } else {
        palette::DARK_GRAY
    };
    super::bordered_rect(area.x, area.y, area.w, area.h, fill, border);

    let text_x = area.x + 3.0;
    let text_y = area.y + (area.h - 8.0) / 2.0;

    let mut text = value.to_string();

    if active {
        // Collect typed chars
        for ch in crate::input::typed_chars() {
            text.push(ch);
        }
        if crate::input::backspace_pressed() {
            text.pop();
        }

        // Draw text
        let max_chars = ((area.w - 6.0) / 8.0) as usize;
        let visible = if text.len() > max_chars {
            &text[text.len() - max_chars..]
        } else {
            &text
        };
        draw_pixel_text(font, visible, text_x, text_y, palette::WHITE);

        // Blinking cursor
        let cursor_phase = (get_time() * 3.0) as u32 % 2;
        if cursor_phase == 0 {
            let cursor_x = text_x + visible.len() as f32 * 8.0;
            draw_rectangle(cursor_x, text_y, 1.0, 8.0, palette::YELLOW);
        }
    } else if text.is_empty() {
        // Placeholder
        draw_pixel_text(font, placeholder, text_x, text_y, palette::DARK_GRAY);
    } else {
        let max_chars = ((area.w - 6.0) / 8.0) as usize;
        let visible = if text.len() > max_chars {
            &text[..max_chars]
        } else {
            &text
        };
        draw_pixel_text(font, visible, text_x, text_y, palette::LIGHT_GRAY);
    }

    text
}
