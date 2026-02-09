//! Scrollable selectable list widget.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::font::{PixelFont, draw_pixel_text};
use crate::theme::palette;

/// Height of one list row in pixels.
pub const ROW_HEIGHT: f32 = 12.0;

/// Draw a scrollable list and return the index of a clicked item (if any).
///
/// `items` is a slice of (label, color) pairs.
/// `cursor` is the currently selected index.
/// `scroll_offset` is the first visible row.
pub fn draw_list(
    font: &PixelFont,
    items: &[(String, Color)],
    cursor: usize,
    scroll_offset: usize,
    area: &Rect2,
    mouse_x: f32,
    mouse_y: f32,
) -> Option<usize> {
    let visible_rows = (area.h / ROW_HEIGHT) as usize;
    let mut clicked_idx = None;

    for (vi, idx) in (scroll_offset..items.len().min(scroll_offset + visible_rows)).enumerate() {
        let y = area.y + vi as f32 * ROW_HEIGHT;
        let row_area = Rect2::new(area.x, y, area.w, ROW_HEIGHT);

        // Highlight selected row
        if idx == cursor {
            draw_rectangle(area.x, y, area.w, ROW_HEIGHT, palette::DARK_PURPLE);
        }

        // Hover highlight
        let hovered = row_area.contains(mouse_x, mouse_y);
        if hovered && idx != cursor {
            draw_rectangle(
                area.x,
                y,
                area.w,
                ROW_HEIGHT,
                Color::new(1.0, 1.0, 1.0, 0.05),
            );
        }

        // Click detection
        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            clicked_idx = Some(idx);
        }

        let (ref label, color) = items[idx];
        // Truncate label to fit (account for 2px padding + 2px scrollbar margin)
        let max_chars = ((area.w - 6.0) / 8.0) as usize;
        if label.len() > max_chars && max_chars > 3 {
            let mut truncated = label[..max_chars - 3].to_string();
            truncated.push_str("...");
            draw_pixel_text(font, &truncated, area.x + 2.0, y + 2.0, color);
        } else {
            draw_pixel_text(font, label, area.x + 2.0, y + 2.0, color);
        }
    }

    // Scroll bar if needed
    if items.len() > visible_rows {
        let bar_h = (visible_rows as f32 / items.len() as f32) * area.h;
        let bar_y = area.y + (scroll_offset as f32 / items.len() as f32) * area.h;
        draw_rectangle(area.x + area.w - 2.0, bar_y, 2.0, bar_h, palette::DARK_GRAY);
    }

    clicked_idx
}

/// Compute the scroll offset to keep the cursor visible.
pub fn scroll_to_cursor(cursor: usize, scroll_offset: usize, visible_rows: usize) -> usize {
    if cursor < scroll_offset {
        cursor
    } else if cursor >= scroll_offset + visible_rows {
        cursor - visible_rows + 1
    } else {
        scroll_offset
    }
}
