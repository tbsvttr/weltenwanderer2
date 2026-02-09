//! World explorer screen: entity list (left) + entity detail (right).

use macroquad::prelude::*;

use ww_core::entity::EntityKind;

use crate::app::AppState;
use crate::theme::font::draw_pixel_text;
use crate::theme::palette;
use crate::theme::{CANVAS_H, CANVAS_W, kind_color, mouse_canvas_position};
use crate::widget::Rect2;
use crate::widget::list::{self, ROW_HEIGHT};
use crate::widget::panel::{draw_panel, draw_panel_titled};
use crate::widget::text_area;

use super::{Screen, ScreenId, Transition};

/// Explorer screen state.
pub struct ExplorerScreen {
    /// Scroll offset for the entity list.
    pub list_scroll: usize,
    /// Scroll offset for the detail text area.
    pub detail_scroll: usize,
    /// Whether the search input is active.
    pub search_active: bool,
    /// Key repeat tracker for Up arrow.
    key_up: crate::input::KeyRepeat,
    /// Key repeat tracker for Down arrow.
    key_down: crate::input::KeyRepeat,
    /// Key repeat tracker for PageUp.
    key_pgup: crate::input::KeyRepeat,
    /// Key repeat tracker for PageDown.
    key_pgdn: crate::input::KeyRepeat,
}

impl Default for ExplorerScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplorerScreen {
    /// Create a new explorer screen.
    pub fn new() -> Self {
        Self {
            list_scroll: 0,
            detail_scroll: 0,
            search_active: false,
            key_up: crate::input::KeyRepeat::new(),
            key_down: crate::input::KeyRepeat::new(),
            key_pgup: crate::input::KeyRepeat::new(),
            key_pgdn: crate::input::KeyRepeat::new(),
        }
    }
}

impl Screen for ExplorerScreen {
    fn update(&mut self, app: &mut AppState) -> Transition {
        if is_key_pressed(KeyCode::Escape) {
            if self.search_active {
                self.search_active = false;
                app.search_query.clear();
            } else if app.selected_entity.is_some() {
                app.selected_entity = None;
                self.detail_scroll = 0;
            } else {
                return Transition::Pop;
            }
            return Transition::None;
        }

        // Tab to switch views
        if crate::input::tab_pressed() {
            return Transition::Replace(ScreenId::Graph);
        }

        // Mouse: tab click
        let (mx, my) = crate::theme::mouse_canvas_position();
        if let Some(t) = super::handle_tab_click(mx, my, 0) {
            return t;
        }

        // Search toggle
        if is_key_pressed(KeyCode::Slash) && !self.search_active {
            self.search_active = true;
            app.search_query.clear();
            return Transition::None;
        }

        if self.search_active {
            for ch in crate::input::typed_chars() {
                if ch != '/' {
                    app.search_query.push(ch);
                    app.list_cursor = 0;
                    self.list_scroll = 0;
                }
            }
            if crate::input::backspace_pressed() {
                app.search_query.pop();
                app.list_cursor = 0;
                self.list_scroll = 0;
            }
            if crate::input::enter_pressed() {
                self.search_active = false;
            }
            return Transition::None;
        }

        let items = self.filtered_items(app);

        // Navigation (with key repeat for held keys)
        if self.key_up.check(KeyCode::Up) {
            app.list_cursor = app.list_cursor.saturating_sub(1);
        }
        if self.key_down.check(KeyCode::Down) && !items.is_empty() {
            app.list_cursor = (app.list_cursor + 1).min(items.len() - 1);
        }

        // Select entity (Enter key)
        if crate::input::enter_pressed()
            && !items.is_empty()
            && let Some(world) = &app.world
        {
            let entity_name = &items[app.list_cursor].0;
            if let Some(entity) = world.find_by_name(entity_name) {
                app.selected_entity = Some(entity.id);
                self.detail_scroll = 0;
            }
        }

        // Select entity (mouse click on list)
        if is_mouse_button_pressed(MouseButton::Left) && mx < CANVAS_W * 0.5 && my > 33.0 {
            let visible_rows = ((CANVAS_H - 28.0 - 20.0) / ROW_HEIGHT) as usize;
            let scroll = list::scroll_to_cursor(app.list_cursor, self.list_scroll, visible_rows);
            let row = ((my - 33.0) / ROW_HEIGHT) as usize + scroll;
            if row < items.len() {
                app.list_cursor = row;
                if let Some(world) = &app.world
                    && let Some(entity) = world.find_by_name(&items[row].0)
                {
                    app.selected_entity = Some(entity.id);
                    self.detail_scroll = 0;
                }
            }
        }

        // Scroll detail with page up/down (with key repeat)
        if self.key_pgdn.check(KeyCode::PageDown) {
            self.detail_scroll = self.detail_scroll.saturating_add(5);
        }
        if self.key_pgup.check(KeyCode::PageUp) {
            self.detail_scroll = self.detail_scroll.saturating_sub(5);
        }

        // Mouse scroll wheel — scroll list or detail depending on mouse position
        let scroll_delta = crate::input::scroll_y();
        if scroll_delta != 0.0 {
            let (mx, _my) = crate::theme::mouse_canvas_position();
            let mid_x = CANVAS_W * 0.5;
            if mx < mid_x {
                // Scroll entity list
                if scroll_delta > 0.0 {
                    app.list_cursor = app.list_cursor.saturating_sub(3);
                } else {
                    let max = if items.is_empty() { 0 } else { items.len() - 1 };
                    app.list_cursor = (app.list_cursor + 3).min(max);
                }
            } else {
                // Scroll detail panel
                if scroll_delta > 0.0 {
                    self.detail_scroll = self.detail_scroll.saturating_sub(3);
                } else {
                    self.detail_scroll = self.detail_scroll.saturating_add(3);
                }
            }
        }

        Transition::None
    }

    fn draw(&self, app: &AppState) {
        let (mx, my) = mouse_canvas_position();

        // Tab bar
        let tab_area = Rect2::new(0.0, 0.0, CANVAS_W, 14.0);
        crate::widget::tabs::draw_tabs(&app.font, super::TAB_LABELS, 0, &tab_area, mx, my);

        // Content area: 2px gap below tab bar, 12px reserved for status bar
        let full = Rect2::new(0.0, 16.0, CANVAS_W, CANVAS_H - 28.0);

        // Split: 50% list, 50% detail
        let (list_area, detail_area) = full.split_h(0.5);

        // Entity list panel (no title — tab bar already labels it)
        draw_panel(&list_area);
        let inner = list_area.inset(3.0);

        // Search bar at top of list
        let (search_area, list_content) = inner.take_top(14.0);
        if self.search_active || !app.search_query.is_empty() {
            let search_text = format!("/{}", app.search_query);
            draw_pixel_text(
                &app.font,
                &search_text,
                search_area.x + 2.0,
                search_area.y + 3.0,
                if self.search_active {
                    palette::YELLOW
                } else {
                    palette::DARK_GRAY
                },
            );
            crate::widget::draw_separator(search_area.x, search_area.y + 12.0, search_area.w);
        }

        let items = self.filtered_items(app);
        let visible_rows = (list_content.h / ROW_HEIGHT) as usize;
        let scroll = list::scroll_to_cursor(app.list_cursor, self.list_scroll, visible_rows);
        list::draw_list(
            &app.font,
            &items,
            app.list_cursor,
            scroll,
            &list_content,
            mx,
            my,
        );

        // Detail panel
        draw_panel_titled(&detail_area, "Detail", &app.font);
        // Start content below the panel title (title occupies ~12px from top border)
        let detail_inner = Rect2::new(
            detail_area.x + 4.0,
            detail_area.y + 14.0,
            detail_area.w - 8.0,
            detail_area.h - 18.0,
        );

        if let Some(entity_id) = app.selected_entity
            && let Some(world) = &app.world
            && let Some(entity) = world.get_entity(entity_id)
        {
            // Entity name (full width, line 1)
            let kind_str = format!("{}", entity.kind);
            draw_pixel_text(
                &app.font,
                &entity.name,
                detail_inner.x,
                detail_inner.y,
                palette::YELLOW,
            );

            // Sprite icon + kind label (line 2)
            let icon_texture = app.sprites.for_kind(&kind_str);
            draw_texture_ex(
                icon_texture,
                detail_inner.x,
                detail_inner.y + 11.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(12.0, 12.0)),
                    ..Default::default()
                },
            );
            draw_pixel_text(
                &app.font,
                &kind_str,
                detail_inner.x + 16.0,
                detail_inner.y + 13.0,
                kind_color(&kind_str),
            );

            // Description
            let text_area = Rect2::new(
                detail_inner.x,
                detail_inner.y + 26.0,
                detail_inner.w,
                detail_inner.h - 26.0,
            );

            let mut content = String::new();
            if !entity.description.is_empty() {
                content.push_str(&entity.description);
                content.push_str("\n\n");
            }

            // Properties
            if !entity.properties.is_empty() {
                content.push_str("**Properties**\n");
                let mut props: Vec<_> = entity.properties.iter().collect();
                props.sort_by_key(|(k, _)| k.as_str());
                for (key, value) in props {
                    content.push_str(&format!("  {key}: {value}\n"));
                }
                content.push('\n');
            }

            // Relationships
            let rels = world.relationships_of(entity_id);
            if !rels.is_empty() {
                content.push_str("**Relationships**\n");
                for rel in &rels {
                    let other_id = if rel.source == entity_id {
                        rel.target
                    } else {
                        rel.source
                    };
                    if let Some(other) = world.get_entity(other_id) {
                        let label = rel.label.as_deref().unwrap_or("");
                        content.push_str(&format!("  {} {} {}\n", rel.kind, label, other.name));
                    }
                }
            }

            text_area::draw_text_area(&app.font, &content, self.detail_scroll, &text_area);
        } else {
            draw_pixel_text(
                &app.font,
                "Select an entity",
                detail_inner.x + 4.0,
                detail_inner.y + 4.0,
                palette::DARK_GRAY,
            );
        }

        // Status bar background + text
        draw_rectangle(0.0, CANVAS_H - 12.0, CANVAS_W, 12.0, palette::BLACK);
        let status = if let Some(world) = &app.world {
            format!("{} | Scroll | /search | Tab:graph | Esc", world.meta.name,)
        } else {
            "No world | Esc:back".to_string()
        };
        draw_pixel_text(&app.font, &status, 2.0, CANVAS_H - 10.0, palette::DARK_GRAY);
    }
}

impl ExplorerScreen {
    fn filtered_items(&self, app: &AppState) -> Vec<(String, Color)> {
        let Some(world) = &app.world else {
            return Vec::new();
        };

        let query_lower = app.search_query.to_lowercase();
        let mut items: Vec<(String, Color)> = world
            .all_entities()
            .filter(|e| {
                if query_lower.is_empty() {
                    true
                } else {
                    e.name.to_lowercase().contains(&query_lower)
                }
            })
            .map(|e| {
                let kind_str = match &e.kind {
                    EntityKind::Custom(s) => s.as_str(),
                    other => match other {
                        EntityKind::Location => "location",
                        EntityKind::Character => "character",
                        EntityKind::Faction => "faction",
                        EntityKind::Event => "event",
                        EntityKind::Item => "item",
                        EntityKind::Lore => "lore",
                        EntityKind::Custom(_) => unreachable!(),
                    },
                };
                let color = kind_color(kind_str);
                (e.name.clone(), color)
            })
            .collect();
        items.sort_by(|(a, _), (b, _)| a.to_lowercase().cmp(&b.to_lowercase()));
        items
    }
}
