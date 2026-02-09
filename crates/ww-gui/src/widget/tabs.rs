//! Tab row widget for switching between views.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::font::{PixelFont, draw_pixel_text, measure_text_width};
use crate::theme::palette;

/// Draw a tab row and return the index of a clicked tab (if any).
///
/// `labels` are the tab names, `active` is the currently selected index.
pub fn draw_tabs(
    font: &PixelFont,
    labels: &[&str],
    active: usize,
    area: &Rect2,
    mouse_x: f32,
    mouse_y: f32,
) -> Option<usize> {
    let tab_w = area.w / labels.len() as f32;
    let mut clicked = None;

    // Background
    draw_rectangle(area.x, area.y, area.w, area.h, palette::BLACK);

    for (i, label) in labels.iter().enumerate() {
        let tx = area.x + i as f32 * tab_w;
        let tab = Rect2::new(tx, area.y, tab_w, area.h);
        let hovered = tab.contains(mouse_x, mouse_y);

        if i == active {
            draw_rectangle(tx, area.y, tab_w, area.h, palette::DARK_BLUE);
            draw_rectangle(tx, area.y + area.h - 2.0, tab_w, 2.0, palette::YELLOW);
        } else if hovered {
            draw_rectangle(tx, area.y, tab_w, area.h, Color::new(0.1, 0.1, 0.2, 1.0));
        }

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            clicked = Some(i);
        }

        let text_w = measure_text_width(label);
        let text_x = tx + (tab_w - text_w) / 2.0;
        let color = if i == active {
            palette::YELLOW
        } else {
            palette::LIGHT_GRAY
        };
        draw_pixel_text(font, label, text_x, area.y + (area.h - 8.0) / 2.0, color);
    }

    // Bottom border
    draw_rectangle(
        area.x,
        area.y + area.h - 1.0,
        area.w,
        1.0,
        palette::DARK_GRAY,
    );

    clicked
}
