//! Solo TTRPG session screen with oracle, scenes, and journal sidebar.

use macroquad::prelude::*;

use ww_solo::SoloSession;
use ww_solo::config::SoloConfig;

use crate::app::AppState;
use crate::theme::font::draw_pixel_text;
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W, mouse_canvas_position};
use crate::widget::Rect2;
use crate::widget::panel::{draw_panel, draw_panel_titled};
use crate::widget::text_area;

use super::{Screen, Transition};

/// Solo session screen state.
pub struct SoloScreen {
    /// The solo session.
    session: Option<SoloSession>,
    /// Text output log.
    output_log: String,
    /// Current input text.
    input_text: String,
    /// Scroll offset for the output area.
    scroll: usize,
    /// Initialization error.
    error: Option<String>,
}

impl Default for SoloScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SoloScreen {
    /// Create a new solo screen.
    pub fn new() -> Self {
        Self {
            session: None,
            output_log: String::new(),
            input_text: String::new(),
            scroll: 0,
            error: None,
        }
    }

    fn initialize(&mut self, app: &AppState) {
        let Some(world) = &app.world else {
            self.error = Some("No world loaded".to_string());
            return;
        };

        match SoloSession::new(world.clone(), SoloConfig::default()) {
            Ok(mut session) => {
                // Initial look
                match session.process("look") {
                    Ok(output) => {
                        self.output_log = output;
                    }
                    Err(e) => {
                        self.output_log = format!("Error: {e}");
                    }
                }
                self.session = Some(session);
            }
            Err(e) => {
                self.error = Some(format!("{e}"));
            }
        }
    }
}

impl Screen for SoloScreen {
    fn update(&mut self, app: &mut AppState) -> Transition {
        if self.session.is_none() && self.error.is_none() {
            self.initialize(app);
        }

        if is_key_pressed(KeyCode::Escape) {
            return Transition::Pop;
        }

        // Mouse: tab click
        let (mx, my) = mouse_canvas_position();
        if let Some(t) = super::handle_tab_click(mx, my, 4) {
            return t;
        }

        // Text input
        for ch in crate::input::typed_chars() {
            self.input_text.push(ch);
        }
        if crate::input::backspace_pressed() {
            self.input_text.pop();
        }

        // Submit command
        if crate::input::enter_pressed()
            && !self.input_text.is_empty()
            && let Some(session) = &mut self.session
        {
            let input = self.input_text.clone();
            self.input_text.clear();

            self.output_log.push_str(&format!("\n\n> {input}\n\n"));

            match session.process(&input) {
                Ok(output) => {
                    self.output_log.push_str(&output);
                }
                Err(e) => {
                    self.output_log.push_str(&format!("Error: {e}"));
                }
            }

            self.scroll = usize::MAX;
        }

        if is_key_pressed(KeyCode::PageUp) {
            self.scroll = self.scroll.saturating_sub(5);
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.scroll = self.scroll.saturating_add(5);
        }
        let wheel = crate::input::scroll_y();
        if wheel > 0.0 {
            self.scroll = self.scroll.saturating_sub(3);
        } else if wheel < 0.0 {
            self.scroll = self.scroll.saturating_add(3);
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = mouse_canvas_position();

        // Tab bar
        let tab_area = Rect2::new(0.0, 0.0, CANVAS_W, 14.0);
        crate::widget::tabs::draw_tabs(&app.font, super::TAB_LABELS, 4, &tab_area, mx, my);

        let full = Rect2::new(0.0, 16.0, CANVAS_W, CANVAS_H - 36.0);
        let (main_area, sidebar_area) = full.split_h(0.7);

        // Main output panel
        let output_panel = Rect2::new(
            main_area.x + 2.0,
            main_area.y + 2.0,
            main_area.w - 4.0,
            main_area.h - 20.0,
        );
        draw_panel(&output_panel);
        let output_inner = output_panel.inset(3.0);

        if let Some(ref err) = self.error {
            draw_pixel_text(&app.font, err, output_inner.x, output_inner.y, palette::RED);
        } else {
            let total =
                text_area::draw_text_area(&app.font, &self.output_log, self.scroll, &output_inner);
            if self.scroll == usize::MAX {
                let visible = (output_inner.h / 10.0) as usize;
                let _ = text_area::draw_text_area(
                    &app.font,
                    &self.output_log,
                    total.saturating_sub(visible),
                    &output_inner,
                );
            }
        }

        // Input
        let input_area = Rect2::new(
            main_area.x + 2.0,
            main_area.y + main_area.h - 16.0,
            main_area.w - 4.0,
            14.0,
        );
        crate::widget::input::draw_input(
            &app.font,
            &self.input_text,
            "type a command...",
            true,
            &input_area,
        );

        // Sidebar: status panel
        let sidebar = Rect2::new(
            sidebar_area.x + 2.0,
            sidebar_area.y + 2.0,
            sidebar_area.w - 4.0,
            sidebar_area.h,
        );
        draw_panel_titled(&sidebar, "Session", &app.font);
        // Start content below the panel title
        let sb_inner = Rect2::new(
            sidebar.x + 4.0,
            sidebar.y + 14.0,
            sidebar.w - 8.0,
            sidebar.h - 18.0,
        );

        if let Some(session) = &self.session {
            let mut y = sb_inner.y;

            // Chaos factor
            let chaos_text = format!("Chaos: {}/9", session.chaos().value());
            draw_pixel_text(&app.font, &chaos_text, sb_inner.x, y, palette::ORANGE);
            y += 12.0;

            // Scene
            let scene_text = if let Some(scene) = session.current_scene() {
                format!("Scene #{}", scene.number)
            } else {
                "No active scene".to_string()
            };
            draw_pixel_text(&app.font, &scene_text, sb_inner.x, y, palette::LIGHT_GRAY);
            y += 16.0;

            // Threads
            draw_pixel_text(&app.font, "Threads:", sb_inner.x, y, palette::YELLOW);
            y += 10.0;
            let threads = session.threads().active();
            if threads.is_empty() {
                draw_pixel_text(&app.font, "  (none)", sb_inner.x, y, palette::DARK_GRAY);
                y += 10.0;
            } else {
                for t in threads {
                    draw_pixel_text(
                        &app.font,
                        &format!("  {}", t.name),
                        sb_inner.x,
                        y,
                        palette::LIGHT_GRAY,
                    );
                    y += 10.0;
                }
            }

            y += 6.0;

            // NPCs
            draw_pixel_text(&app.font, "NPCs:", sb_inner.x, y, palette::YELLOW);
            y += 10.0;
            let npcs = session.npcs().list();
            if npcs.is_empty() {
                draw_pixel_text(&app.font, "  (none)", sb_inner.x, y, palette::DARK_GRAY);
            } else {
                for n in npcs {
                    draw_pixel_text(
                        &app.font,
                        &format!("  {}", n.name),
                        sb_inner.x,
                        y,
                        palette::LIGHT_GRAY,
                    );
                    y += 10.0;
                }
            }
        }

        // Status bar
        draw_pixel_text(
            &app.font,
            "Enter: send | help: commands | Esc: back",
            2.0,
            CANVAS_H - 10.0,
            palette::DARK_GRAY,
        );
    }
}
