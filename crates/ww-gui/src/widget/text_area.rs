//! Scrollable multi-line text area with word wrap.

use macroquad::prelude::*;

use super::Rect2;
use crate::theme::font::{PixelFont, draw_pixel_text};
use crate::theme::palette;

/// Line height in the text area.
const LINE_HEIGHT: f32 = 10.0;

/// Draw a scrollable text area with word-wrapped content.
///
/// Returns the total number of wrapped lines (for scroll calculations).
pub fn draw_text_area(font: &PixelFont, text: &str, scroll_offset: usize, area: &Rect2) -> usize {
    let chars_per_line = ((area.w - 4.0) / 8.0) as usize;
    if chars_per_line == 0 {
        return 0;
    }

    let wrapped = wrap_text(text, chars_per_line);
    let visible_lines = (area.h / LINE_HEIGHT) as usize;

    let end = wrapped
        .len()
        .min(scroll_offset.saturating_add(visible_lines));
    for (vi, line_idx) in (scroll_offset..end).enumerate() {
        let y = area.y + vi as f32 * LINE_HEIGHT;
        let line = &wrapped[line_idx];

        // Check for **bold** markers
        let mut x = area.x + 2.0;
        let mut remaining = line.as_str();
        while !remaining.is_empty() {
            if let Some(bold_start) = remaining.find("**") {
                // Draw text before bold
                let before = &remaining[..bold_start];
                if !before.is_empty() {
                    draw_pixel_text(font, before, x, y, palette::LIGHT_GRAY);
                    x += before.len() as f32 * 8.0;
                }
                remaining = &remaining[bold_start + 2..];
                // Find closing **
                if let Some(bold_end) = remaining.find("**") {
                    let bold_text = &remaining[..bold_end];
                    draw_pixel_text(font, bold_text, x, y, palette::YELLOW);
                    x += bold_text.len() as f32 * 8.0;
                    remaining = &remaining[bold_end + 2..];
                } else {
                    // No closing **, just draw rest
                    draw_pixel_text(font, remaining, x, y, palette::LIGHT_GRAY);
                    break;
                }
            } else {
                draw_pixel_text(font, remaining, x, y, palette::LIGHT_GRAY);
                break;
            }
        }
    }

    wrapped.len()
}

/// Word-wrap text into lines of at most `max_chars` characters.
pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let words: Vec<&str> = raw_line.split_whitespace().collect();
        let mut current = String::new();

        for word in words {
            if current.is_empty() {
                if word.len() > max_chars {
                    // Force-break long words
                    for chunk in word.as_bytes().chunks(max_chars) {
                        lines.push(String::from_utf8_lossy(chunk).to_string());
                    }
                } else {
                    current = word.to_string();
                }
            } else if current.len() + 1 + word.len() <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
