//! Visual dice roller screen with pool configuration and roll animation.

use ::rand::SeedableRng;
use ::rand::rngs::StdRng;
use macroquad::prelude::*;

use ww_mechanics::{DicePool, Die, RollResult};

use crate::app::AppState;
use crate::theme::font::draw_pixel_text;
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W};
use crate::widget::Rect2;
use crate::widget::button::draw_button;
use crate::widget::panel::draw_panel_titled;

use super::{Screen, Transition};

/// Dice roller screen state.
pub struct DiceScreen {
    /// Number of dice in the pool.
    pool_size: u32,
    /// Die type (sides).
    die_sides: u32,
    /// Last roll result.
    result: Option<RollResult>,
    /// RNG used for rolling.
    rng: StdRng,
}

impl Default for DiceScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl DiceScreen {
    /// Create a new dice screen.
    pub fn new() -> Self {
        Self {
            pool_size: 2,
            die_sides: 20,
            result: None,
            rng: StdRng::seed_from_u64(42),
        }
    }
}

/// Convert a sides count to a `Die` variant.
fn die_from_sides(sides: u32) -> Die {
    match sides {
        4 => Die::D4,
        6 => Die::D6,
        8 => Die::D8,
        10 => Die::D10,
        12 => Die::D12,
        20 => Die::D20,
        100 => Die::D100,
        n => Die::Custom(n),
    }
}

impl DiceScreen {
    /// Roll the current dice pool.
    fn roll(&mut self) {
        let die = die_from_sides(self.die_sides);
        let pool = DicePool::new().add(die, self.pool_size);
        self.result = Some(pool.roll(&mut self.rng));
    }
}

impl Screen for DiceScreen {
    fn update(&mut self, _app: &mut AppState) -> Transition {
        if is_key_pressed(KeyCode::Escape) {
            return Transition::Pop;
        }

        let (mx, my) = crate::theme::mouse_canvas_position();

        if is_mouse_button_pressed(MouseButton::Left) {
            // Match the layout from draw()
            let panel = Rect2::new(40.0, 20.0, CANVAS_W - 80.0, CANVAS_H - 40.0);
            let inner = Rect2::new(
                panel.x + 6.0,
                panel.y + 14.0,
                panel.w - 12.0,
                panel.h - 20.0,
            );

            // Die type buttons row (y = inner.y + 16)
            let btn_y = inner.y + 16.0;
            let die_types: [(u32, f32); 6] = [
                (4, 0.0),
                (6, 44.0),
                (8, 88.0),
                (10, 132.0),
                (12, 176.0),
                (20, 220.0),
            ];
            for (sides, offset) in &die_types {
                let area = Rect2::new(inner.x + offset, btn_y, 40.0, 14.0);
                if area.contains(mx, my) {
                    self.die_sides = *sides;
                }
            }

            // Pool size row (y = inner.y + 38)
            let pool_y = inner.y + 38.0;
            let minus_area = Rect2::new(inner.x, pool_y, 30.0, 14.0);
            if minus_area.contains(mx, my) {
                self.pool_size = self.pool_size.saturating_sub(1).max(1);
            }

            let plus_area = Rect2::new(inner.x + 60.0, pool_y, 30.0, 14.0);
            if plus_area.contains(mx, my) {
                self.pool_size = (self.pool_size + 1).min(20);
            }

            // Roll button
            let roll_area = Rect2::new(inner.x + 110.0, pool_y, 60.0, 14.0);
            if roll_area.contains(mx, my) {
                self.roll();
            }
        }

        // Keyboard shortcut: Space to roll
        if is_key_pressed(KeyCode::Space) {
            self.roll();
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = crate::theme::mouse_canvas_position();

        let panel = Rect2::new(40.0, 20.0, CANVAS_W - 80.0, CANVAS_H - 40.0);
        draw_panel_titled(&panel, "Dice Roller", &app.font);
        // Start content below the panel title
        let inner = Rect2::new(
            panel.x + 6.0,
            panel.y + 14.0,
            panel.w - 12.0,
            panel.h - 20.0,
        );

        let mut y = inner.y;

        // Pool size controls
        draw_pixel_text(
            &app.font,
            &format!("Pool: {}d{}", self.pool_size, self.die_sides),
            inner.x,
            y,
            palette::YELLOW,
        );
        y += 16.0;

        // Die type buttons
        let die_types = [
            ("d4", 4),
            ("d6", 6),
            ("d8", 8),
            ("d10", 10),
            ("d12", 12),
            ("d20", 20),
        ];
        let btn_w = 40.0;
        let mut bx = inner.x;
        for (label, _sides) in &die_types {
            let area = Rect2::new(bx, y, btn_w, 14.0);
            draw_button(&app.font, label, &area, mx, my);
            bx += btn_w + 4.0;
        }
        y += 22.0;

        // Pool size buttons
        let minus_area = Rect2::new(inner.x, y, 30.0, 14.0);
        draw_button(&app.font, " - ", &minus_area, mx, my);

        draw_pixel_text(
            &app.font,
            &format!(" {} ", self.pool_size),
            inner.x + 34.0,
            y + 3.0,
            palette::WHITE,
        );

        let plus_area = Rect2::new(inner.x + 60.0, y, 30.0, 14.0);
        draw_button(&app.font, " + ", &plus_area, mx, my);

        // Roll button
        let roll_area = Rect2::new(inner.x + 110.0, y, 60.0, 14.0);
        draw_button(&app.font, "ROLL!", &roll_area, mx, my);
        y += 22.0;

        // Result display
        if let Some(result) = &self.result {
            draw_pixel_text(&app.font, "Result:", inner.x, y, palette::LIGHT_GRAY);
            y += 12.0;

            // Individual dice
            for (i, die_result) in result.dice.iter().enumerate() {
                let dx = inner.x + (i as f32 * 36.0) % inner.w;
                let dy = y + (i as f32 * 36.0 / inner.w).floor() * 30.0;

                // Draw a die face
                draw_rectangle(dx, dy, 24.0, 24.0, palette::WHITE);
                draw_rectangle(dx + 1.0, dy + 1.0, 22.0, 22.0, palette::DARK_BLUE);
                draw_pixel_text(
                    &app.font,
                    &format!("{}", die_result.value),
                    dx + 4.0,
                    dy + 8.0,
                    palette::YELLOW,
                );
            }

            let dice_rows = ((result.dice.len() as f32 * 36.0) / inner.w).ceil() as usize;
            y += dice_rows.max(1) as f32 * 30.0 + 4.0;

            // Total
            draw_pixel_text(
                &app.font,
                &format!("Total: {}", result.total()),
                inner.x,
                y,
                palette::GREEN,
            );
        }

        // Status
        draw_pixel_text(
            &app.font,
            "Esc: back",
            2.0,
            CANVAS_H - 10.0,
            palette::DARK_GRAY,
        );
    }
}
