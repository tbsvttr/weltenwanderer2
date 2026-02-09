//! Progress/track bar widget for HP, Ruin, Honor, etc.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::font::{PixelFont, draw_pixel_text};
use crate::theme::palette;

/// Draw a labeled progress bar.
///
/// `fraction` should be 0.0..=1.0. The bar shows `label: current/max`.
pub fn draw_bar(
    font: &PixelFont,
    label: &str,
    current: i32,
    max: i32,
    fraction: f32,
    area: &Rect2,
    fill_color: Color,
) {
    // Label
    draw_pixel_text(font, label, area.x, area.y, palette::LIGHT_GRAY);

    // Bar background
    let bar_x = area.x;
    let bar_y = area.y + 10.0;
    let bar_w = area.w;
    let bar_h = area.h - 10.0;
    draw_rectangle(bar_x, bar_y, bar_w, bar_h, palette::BLACK);
    draw_rectangle(bar_x, bar_y, bar_w, 1.0, palette::DARK_GRAY);
    draw_rectangle(bar_x, bar_y + bar_h - 1.0, bar_w, 1.0, palette::DARK_GRAY);
    draw_rectangle(bar_x, bar_y, 1.0, bar_h, palette::DARK_GRAY);
    draw_rectangle(bar_x + bar_w - 1.0, bar_y, 1.0, bar_h, palette::DARK_GRAY);

    // Fill
    let fill_w = (bar_w - 2.0) * fraction.clamp(0.0, 1.0);
    if fill_w > 0.0 {
        draw_rectangle(bar_x + 1.0, bar_y + 1.0, fill_w, bar_h - 2.0, fill_color);
    }

    // Value text
    let value_text = format!("{current}/{max}");
    let text_w = crate::theme::font::measure_text_width(&value_text);
    let text_x = bar_x + (bar_w - text_w) / 2.0;
    draw_pixel_text(font, &value_text, text_x, bar_y + 1.0, palette::WHITE);
}
