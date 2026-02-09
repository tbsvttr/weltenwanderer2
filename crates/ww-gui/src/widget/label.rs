//! Single-line text label widget.

use macroquad::prelude::*;

use crate::theme::font::{PixelFont, draw_pixel_text};

/// Draw a label at the given position.
pub fn draw_label(font: &PixelFont, text: &str, x: f32, y: f32, color: Color) {
    draw_pixel_text(font, text, x, y, color);
}

/// Draw a label centered horizontally within a width.
pub fn draw_label_centered(font: &PixelFont, text: &str, x: f32, y: f32, w: f32, color: Color) {
    let text_w = crate::theme::font::measure_text_width(text);
    let cx = x + (w - text_w) / 2.0;
    draw_pixel_text(font, text, cx.max(x), y, color);
}

/// Draw a label right-aligned within a width.
pub fn draw_label_right(font: &PixelFont, text: &str, x: f32, y: f32, w: f32, color: Color) {
    let text_w = crate::theme::font::measure_text_width(text);
    let rx = x + w - text_w;
    draw_pixel_text(font, text, rx.max(x), y, color);
}
