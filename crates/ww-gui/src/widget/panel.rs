//! 9-slice pixel art panel with border and fill.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::palette;

/// Draw a panel with pixel art borders.
pub fn draw_panel(area: &Rect2) {
    super::double_bordered_rect(
        area.x,
        area.y,
        area.w,
        area.h,
        palette::DARK_BLUE,
        palette::LIGHT_GRAY,
    );
}

/// Draw a highlighted panel (selected state).
pub fn draw_panel_highlighted(area: &Rect2) {
    super::double_bordered_rect(
        area.x,
        area.y,
        area.w,
        area.h,
        palette::DARK_BLUE,
        palette::YELLOW,
    );
}

/// Draw a dark/inactive panel.
pub fn draw_panel_dark(area: &Rect2) {
    super::double_bordered_rect(
        area.x,
        area.y,
        area.w,
        area.h,
        palette::BLACK,
        palette::DARK_GRAY,
    );
}

/// Draw a panel with a title inside the top area.
///
/// The title occupies the first 12px inside the panel. Content should
/// start at `area.y + 14.0` or use a suitable top offset.
pub fn draw_panel_titled(area: &Rect2, title: &str, font: &crate::theme::font::PixelFont) {
    draw_panel(area);
    let title_w = crate::theme::font::measure_text_width(title);
    let title_x = area.x + (area.w - title_w) / 2.0;
    // Title bar inside the panel
    draw_rectangle(
        area.x + 2.0,
        area.y + 2.0,
        area.w - 4.0,
        12.0,
        palette::BLACK,
    );
    crate::theme::font::draw_pixel_text(font, title, title_x, area.y + 4.0, palette::WHITE);
}
