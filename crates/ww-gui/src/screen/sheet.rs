//! Character sheet viewer with track bars.

use macroquad::prelude::*;

use crate::app::AppState;
use crate::theme::font::draw_pixel_text;
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W};
use crate::widget::Rect2;
use crate::widget::bar::draw_bar;
use crate::widget::panel::draw_panel_titled;

use super::{Screen, Transition};

/// Character sheet screen state.
pub struct SheetScreen {
    /// The character name to display.
    pub character_name: Option<String>,
}

impl Default for SheetScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetScreen {
    /// Create a new sheet screen.
    pub fn new() -> Self {
        Self {
            character_name: None,
        }
    }
}

impl Screen for SheetScreen {
    fn update(&mut self, _app: &mut AppState) -> Transition {
        if is_key_pressed(KeyCode::Escape) {
            return Transition::Pop;
        }
        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let panel = Rect2::new(20.0, 10.0, CANVAS_W - 40.0, CANVAS_H - 20.0);
        draw_panel_titled(&panel, "Character Sheet", &app.font);
        // Start content below the panel title
        let inner = Rect2::new(
            panel.x + 6.0,
            panel.y + 14.0,
            panel.w - 12.0,
            panel.h - 20.0,
        );

        let Some(world) = &app.world else {
            draw_pixel_text(&app.font, "No world loaded", inner.x, inner.y, palette::RED);
            return;
        };

        // Try to load ruleset and character sheet
        let ruleset = match ww_mechanics::RuleSet::from_world(world) {
            Ok(rs) => rs,
            Err(_) => {
                draw_pixel_text(
                    &app.font,
                    "No ruleset defined",
                    inner.x,
                    inner.y,
                    palette::DARK_GRAY,
                );
                return;
            }
        };

        let entity_id = app.selected_entity;
        let entity = entity_id.and_then(|id| world.get_entity(id));

        let Some(entity) = entity else {
            draw_pixel_text(
                &app.font,
                "Select a character first",
                inner.x,
                inner.y,
                palette::DARK_GRAY,
            );
            return;
        };

        let sheet = match ww_mechanics::CharacterSheet::from_entity(entity, &ruleset) {
            Ok(s) => s,
            Err(e) => {
                draw_pixel_text(
                    &app.font,
                    &format!("Error: {e}"),
                    inner.x,
                    inner.y,
                    palette::RED,
                );
                return;
            }
        };

        let mut y = inner.y;

        // Name
        draw_pixel_text(&app.font, &sheet.name, inner.x, y, palette::YELLOW);
        y += 14.0;

        // Attributes
        draw_pixel_text(&app.font, "Attributes", inner.x, y, palette::LIGHT_GRAY);
        y += 10.0;
        let half_w = inner.w / 2.0;
        let mut col = 0;
        let mut attrs: Vec<_> = sheet.attributes.iter().collect();
        attrs.sort_by_key(|(k, _)| k.to_lowercase());
        for (name, value) in &attrs {
            let x = inner.x + if col == 0 { 4.0 } else { half_w + 4.0 };
            draw_pixel_text(&app.font, &format!("{name}: {value}"), x, y, palette::WHITE);
            col += 1;
            if col >= 2 {
                col = 0;
                y += 10.0;
            }
        }
        if col != 0 {
            y += 10.0;
        }
        y += 6.0;

        // Skills
        draw_pixel_text(&app.font, "Skills", inner.x, y, palette::LIGHT_GRAY);
        y += 10.0;
        col = 0;
        let mut skills: Vec<_> = sheet.skills.iter().collect();
        skills.sort_by_key(|(k, _)| k.to_lowercase());
        for (name, value) in &skills {
            let x = inner.x + if col == 0 { 4.0 } else { half_w + 4.0 };
            draw_pixel_text(&app.font, &format!("{name}: {value}"), x, y, palette::WHITE);
            col += 1;
            if col >= 2 {
                col = 0;
                y += 10.0;
            }
        }
        if col != 0 {
            y += 10.0;
        }
        y += 6.0;

        // Tracks as bars
        let mut tracks: Vec<_> = sheet.tracks.iter().collect();
        tracks.sort_by_key(|(k, _)| k.to_lowercase());
        for (name, track) in &tracks {
            let bar_area = Rect2::new(inner.x + 4.0, y, inner.w - 8.0, 18.0);
            let color = if track.fraction() < 0.3 {
                palette::RED
            } else if track.fraction() < 0.6 {
                palette::ORANGE
            } else {
                palette::GREEN
            };
            draw_bar(
                &app.font,
                name,
                track.current,
                track.max,
                track.fraction() as f32,
                &bar_area,
                color,
            );
            y += 24.0;
        }

        // Focuses
        if !sheet.focuses.is_empty() {
            y += 2.0;
            draw_pixel_text(&app.font, "Focuses", inner.x, y, palette::LIGHT_GRAY);
            y += 10.0;
            for focus in &sheet.focuses {
                draw_pixel_text(&app.font, &format!("  {focus}"), inner.x, y, palette::PEACH);
                y += 10.0;
            }
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
