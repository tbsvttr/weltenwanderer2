//! Chronological timeline view of world events.

use macroquad::prelude::*;

use ww_core::timeline::Timeline;

use crate::app::AppState;
use crate::input::KeyRepeat;
use crate::theme::font::draw_pixel_text;
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W, mouse_canvas_position};
use crate::widget::Rect2;
use crate::widget::panel::draw_panel_titled;

use super::{Screen, ScreenId, Transition};

/// Timeline screen state.
pub struct TimelineScreen {
    /// Cursor position in the event list.
    pub cursor: usize,
    /// Scroll offset.
    pub scroll: usize,
    /// Key repeat tracker for Up arrow.
    key_up: KeyRepeat,
    /// Key repeat tracker for Down arrow.
    key_down: KeyRepeat,
}

impl Default for TimelineScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineScreen {
    /// Create a new timeline screen.
    pub fn new() -> Self {
        Self {
            cursor: 0,
            scroll: 0,
            key_up: KeyRepeat::new(),
            key_down: KeyRepeat::new(),
        }
    }
}

impl Screen for TimelineScreen {
    fn update(&mut self, app: &mut AppState) -> Transition {
        if is_key_pressed(KeyCode::Escape) {
            return Transition::Pop;
        }
        if crate::input::tab_pressed() {
            return Transition::Replace(ScreenId::Explorer);
        }

        // Mouse: tab click
        let (mx, my) = crate::theme::mouse_canvas_position();
        if is_mouse_button_pressed(MouseButton::Left) && my < 14.0 {
            let tab_idx = (mx / (CANVAS_W / 3.0)) as usize;
            match tab_idx {
                0 => return Transition::Replace(ScreenId::Explorer),
                1 => return Transition::Replace(ScreenId::Graph),
                _ => {}
            }
        }

        let count = app
            .world
            .as_ref()
            .map(|w| Timeline::from_world(w).len())
            .unwrap_or(0);

        if self.key_up.check(KeyCode::Up) {
            self.cursor = self.cursor.saturating_sub(1);
        }
        if self.key_down.check(KeyCode::Down) && count > 0 {
            self.cursor = (self.cursor + 1).min(count - 1);
        }

        // Mouse scroll wheel
        let scroll_delta = crate::input::scroll_y();
        if scroll_delta > 0.0 {
            self.cursor = self.cursor.saturating_sub(3);
        } else if scroll_delta < 0.0 && count > 0 {
            self.cursor = (self.cursor + 3).min(count - 1);
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = mouse_canvas_position();

        // Tab bar
        let tab_area = Rect2::new(0.0, 0.0, CANVAS_W, 14.0);
        let tabs = ["Entities", "Graph", "Timeline"];
        crate::widget::tabs::draw_tabs(&app.font, &tabs, 2, &tab_area, mx, my);

        let panel = Rect2::new(4.0, 18.0, CANVAS_W - 8.0, CANVAS_H - 34.0);
        draw_panel_titled(&panel, "Timeline", &app.font);
        // Start content below the panel title (title occupies ~12px from top border)
        let inner = Rect2::new(panel.x + 4.0, panel.y + 14.0, panel.w - 8.0, panel.h - 18.0);

        let Some(world) = &app.world else {
            draw_pixel_text(
                &app.font,
                "No world loaded",
                inner.x,
                inner.y,
                palette::DARK_GRAY,
            );
            return;
        };

        let timeline = Timeline::from_world(world);
        let entries = timeline.entries();

        if entries.is_empty() {
            draw_pixel_text(
                &app.font,
                "No events in world",
                inner.x,
                inner.y,
                palette::DARK_GRAY,
            );
            return;
        }

        let row_h = 20.0;
        let visible_rows = (inner.h / row_h) as usize;
        let scroll = crate::widget::list::scroll_to_cursor(self.cursor, self.scroll, visible_rows);

        for (vi, idx) in (scroll..entries.len().min(scroll + visible_rows)).enumerate() {
            let y = inner.y + vi as f32 * row_h;
            let entry = &entries[idx];

            // Highlight cursor
            if idx == self.cursor {
                draw_rectangle(inner.x, y, inner.w, row_h, palette::DARK_PURPLE);
            }

            // Date
            let date_str = format!("{}", entry.date);
            draw_pixel_text(
                &app.font,
                &date_str,
                inner.x + 2.0,
                y + 2.0,
                palette::ORANGE,
            );

            // Entity name
            draw_pixel_text(
                &app.font,
                &entry.entity.name,
                inner.x + 2.0,
                y + 10.0,
                palette::YELLOW,
            );

            // Era (if present)
            if let Some(era) = &entry.date.era {
                let era_x = inner.x + inner.w - crate::theme::font::measure_text_width(era) - 4.0;
                draw_pixel_text(&app.font, era, era_x, y + 2.0, palette::INDIGO);
            }
        }

        // Status bar
        draw_rectangle(0.0, CANVAS_H - 12.0, CANVAS_W, 12.0, palette::BLACK);
        draw_pixel_text(
            &app.font,
            "Up/Down:nav | Scroll | Tab:entities | Esc:back",
            2.0,
            CANVAS_H - 10.0,
            palette::DARK_GRAY,
        );
    }
}
