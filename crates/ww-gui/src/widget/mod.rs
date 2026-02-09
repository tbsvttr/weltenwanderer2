//! Pixel art widget toolkit for the GUI.
//!
//! Provides reusable UI components: panels, labels, buttons, lists,
//! text areas, input fields, progress bars, and tab rows.

pub mod bar;
pub mod button;
pub mod input;
pub mod label;
pub mod list;
pub mod panel;
pub mod tabs;
pub mod text_area;

use macroquad::prelude::*;

use crate::theme::palette;

/// Draw a filled rectangle.
pub fn filled_rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x, y, w, h, color);
}

/// Draw a 1-pixel bordered rectangle (border inside bounds).
pub fn bordered_rect(x: f32, y: f32, w: f32, h: f32, fill: Color, border: Color) {
    draw_rectangle(x, y, w, h, fill);
    // Top
    draw_rectangle(x, y, w, 1.0, border);
    // Bottom
    draw_rectangle(x, y + h - 1.0, w, 1.0, border);
    // Left
    draw_rectangle(x, y, 1.0, h, border);
    // Right
    draw_rectangle(x + w - 1.0, y, 1.0, h, border);
}

/// Draw a double-border pixel panel (2px border).
pub fn double_bordered_rect(x: f32, y: f32, w: f32, h: f32, fill: Color, border: Color) {
    draw_rectangle(x, y, w, h, fill);
    // Outer border
    draw_rectangle(x, y, w, 1.0, border);
    draw_rectangle(x, y + h - 1.0, w, 1.0, border);
    draw_rectangle(x, y, 1.0, h, border);
    draw_rectangle(x + w - 1.0, y, 1.0, h, border);
    // Inner border (1px inset, lighter)
    let inner = Color::new(
        border.r * 0.7 + fill.r * 0.3,
        border.g * 0.7 + fill.g * 0.3,
        border.b * 0.7 + fill.b * 0.3,
        1.0,
    );
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, 1.0, inner);
    draw_rectangle(x + 1.0, y + h - 2.0, w - 2.0, 1.0, inner);
    draw_rectangle(x + 1.0, y + 1.0, 1.0, h - 2.0, inner);
    draw_rectangle(x + w - 2.0, y + 1.0, 1.0, h - 2.0, inner);
}

/// A simple rectangular area for layout.
#[derive(Debug, Clone, Copy)]
pub struct Rect2 {
    /// X position.
    pub x: f32,
    /// Y position.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect2 {
    /// Create a new rect.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Check if a point is inside this rect.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }

    /// Inset the rect by a margin on all sides.
    pub fn inset(&self, margin: f32) -> Self {
        Self {
            x: self.x + margin,
            y: self.y + margin,
            w: (self.w - margin * 2.0).max(0.0),
            h: (self.h - margin * 2.0).max(0.0),
        }
    }

    /// Split horizontally: returns (left, right) at the given x fraction.
    pub fn split_h(&self, fraction: f32) -> (Self, Self) {
        let left_w = self.w * fraction;
        (
            Self::new(self.x, self.y, left_w, self.h),
            Self::new(self.x + left_w, self.y, self.w - left_w, self.h),
        )
    }

    /// Split vertically: returns (top, bottom) at the given y fraction.
    pub fn split_v(&self, fraction: f32) -> (Self, Self) {
        let top_h = self.h * fraction;
        (
            Self::new(self.x, self.y, self.w, top_h),
            Self::new(self.x, self.y + top_h, self.w, self.h - top_h),
        )
    }

    /// Take a fixed height from the top, return (top_strip, remainder).
    pub fn take_top(&self, height: f32) -> (Self, Self) {
        let h = height.min(self.h);
        (
            Self::new(self.x, self.y, self.w, h),
            Self::new(self.x, self.y + h, self.w, self.h - h),
        )
    }

    /// Take a fixed height from the bottom, return (remainder, bottom_strip).
    pub fn take_bottom(&self, height: f32) -> (Self, Self) {
        let h = height.min(self.h);
        (
            Self::new(self.x, self.y, self.w, self.h - h),
            Self::new(self.x, self.y + self.h - h, self.w, h),
        )
    }
}

/// Horizontal separator line.
pub fn draw_separator(x: f32, y: f32, w: f32) {
    draw_rectangle(x, y, w, 1.0, palette::DARK_GRAY);
}
