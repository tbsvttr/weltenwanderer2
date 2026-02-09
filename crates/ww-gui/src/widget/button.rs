//! Clickable button widget with hover and active states.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::font::{PixelFont, draw_pixel_text, measure_text_width};
use crate::theme::palette;

/// Draw a button and return true if it was clicked this frame.
pub fn draw_button(
    font: &PixelFont,
    label: &str,
    area: &Rect2,
    mouse_x: f32,
    mouse_y: f32,
) -> bool {
    let hovered = area.contains(mouse_x, mouse_y);
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let (fill, border, text_color) = if clicked {
        (palette::DARK_GRAY, palette::WHITE, palette::YELLOW)
    } else if hovered {
        (palette::DARK_BLUE, palette::YELLOW, palette::YELLOW)
    } else {
        (palette::DARK_BLUE, palette::LIGHT_GRAY, palette::WHITE)
    };

    super::bordered_rect(area.x, area.y, area.w, area.h, fill, border);

    let text_w = measure_text_width(label);
    let tx = area.x + (area.w - text_w) / 2.0;
    let ty = area.y + (area.h - 8.0) / 2.0;
    draw_pixel_text(font, label, tx, ty, text_color);

    clicked
}
