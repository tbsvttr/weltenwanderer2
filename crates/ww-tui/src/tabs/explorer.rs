//! Explorer tab: entity list with search and entity detail view.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use ww_core::World;
use ww_core::entity::{Entity, EntityId, EntityKind};

use super::{InputMode, Tab};

/// Sub-view within the explorer tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubView {
    /// Entity list.
    List,
    /// Entity detail.
    Detail,
}

/// Text input mode within the explorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExplorerInput {
    /// Normal vim-like navigation.
    Normal,
    /// Search/filter mode.
    Search,
}

/// Explorer tab state.
pub struct ExplorerTab {
    /// The world data.
    world: World,
    /// Current sub-view.
    sub_view: SubView,
    /// Input mode (normal or search).
    explorer_input: ExplorerInput,

    // List state
    /// Cursor position in the filtered entity list.
    list_cursor: usize,
    /// Optional entity kind filter.
    list_filter: Option<EntityKind>,
    /// Search query string.
    search_query: String,
    /// Filtered entity IDs.
    filtered_ids: Vec<EntityId>,

    // Detail state
    /// Entity ID being viewed in detail.
    detail_entity_id: Option<EntityId>,
    /// Scroll offset in the detail view.
    detail_scroll: u16,

    // Navigation
    /// View stack for back navigation.
    view_stack: Vec<SubView>,
}

impl ExplorerTab {
    /// Create a new explorer tab for the given world.
    pub fn new(world: World) -> Self {
        let mut tab = Self {
            world,
            sub_view: SubView::List,
            explorer_input: ExplorerInput::Normal,
            list_cursor: 0,
            list_filter: None,
            search_query: String::new(),
            filtered_ids: Vec::new(),
            detail_entity_id: None,
            detail_scroll: 0,
            view_stack: Vec::new(),
        };
        tab.update_filtered_list();
        tab
    }

    fn update_filtered_list(&mut self) {
        let mut query = self.world.query();
        if let Some(ref kind) = self.list_filter {
            query = query.kind(kind.clone());
        }
        if !self.search_query.is_empty() {
            query = query.name_contains(&self.search_query);
        }
        self.filtered_ids = query.execute().iter().map(|e| e.id).collect();
        if self.list_cursor >= self.filtered_ids.len() && !self.filtered_ids.is_empty() {
            self.list_cursor = self.filtered_ids.len() - 1;
        }
    }

    fn selected_entity(&self) -> Option<&Entity> {
        self.filtered_ids
            .get(self.list_cursor)
            .and_then(|id| self.world.get_entity(*id))
    }

    fn move_down(&mut self) {
        match self.sub_view {
            SubView::List => {
                if self.list_cursor + 1 < self.filtered_ids.len() {
                    self.list_cursor += 1;
                }
            }
            SubView::Detail => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
        }
    }

    fn move_up(&mut self) {
        match self.sub_view {
            SubView::List => {
                self.list_cursor = self.list_cursor.saturating_sub(1);
            }
            SubView::Detail => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
        }
    }

    fn move_to_top(&mut self) {
        match self.sub_view {
            SubView::List => self.list_cursor = 0,
            SubView::Detail => self.detail_scroll = 0,
        }
    }

    fn move_to_bottom(&mut self) {
        if self.sub_view == SubView::List && !self.filtered_ids.is_empty() {
            self.list_cursor = self.filtered_ids.len() - 1;
        }
    }

    fn select(&mut self) {
        if self.sub_view == SubView::List
            && let Some(entity) = self.selected_entity()
        {
            self.detail_entity_id = Some(entity.id);
            self.detail_scroll = 0;
            self.view_stack.push(self.sub_view);
            self.sub_view = SubView::Detail;
        }
    }

    fn go_back(&mut self) {
        if let Some(prev) = self.view_stack.pop() {
            self.sub_view = prev;
        }
    }
}

impl Tab for ExplorerTab {
    fn input_mode(&self) -> InputMode {
        InputMode::VimNav
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.explorer_input {
            ExplorerInput::Normal => match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.move_down(),
                KeyCode::Char('k') | KeyCode::Up => self.move_up(),
                KeyCode::Char('g') => self.move_to_top(),
                KeyCode::Char('G') => self.move_to_bottom(),
                KeyCode::Enter => self.select(),
                KeyCode::Esc => self.go_back(),
                KeyCode::Char('/') => {
                    self.explorer_input = ExplorerInput::Search;
                    self.search_query.clear();
                }
                _ => {}
            },
            ExplorerInput::Search => match key.code {
                KeyCode::Esc => {
                    self.explorer_input = ExplorerInput::Normal;
                    self.search_query.clear();
                    self.update_filtered_list();
                }
                KeyCode::Enter => {
                    self.explorer_input = ExplorerInput::Normal;
                    self.list_cursor = 0;
                    self.update_filtered_list();
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.list_cursor = 0;
                    self.update_filtered_list();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.list_cursor = 0;
                    self.update_filtered_list();
                }
                _ => {}
            },
        }
        false
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        match mouse.kind {
            MouseEventKind::ScrollUp => match self.sub_view {
                SubView::List => self.list_cursor = self.list_cursor.saturating_sub(1),
                SubView::Detail => self.detail_scroll = self.detail_scroll.saturating_sub(1),
            },
            MouseEventKind::ScrollDown => match self.sub_view {
                SubView::List => {
                    if self.list_cursor + 1 < self.filtered_ids.len() {
                        self.list_cursor += 1;
                    }
                }
                SubView::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_add(1);
                }
            },
            MouseEventKind::Down(MouseButton::Left) => {
                // Handle clicks in list view to select entities
                if self.sub_view == SubView::List {
                    // Row 0 is tab bar, row 1 is block border with title, row 2+ is list content
                    if mouse.row >= 2 {
                        let clicked_row = mouse.row - 2;
                        let target_idx = clicked_row as usize;
                        if target_idx < self.filtered_ids.len() {
                            if target_idx == self.list_cursor {
                                // Double-click effect: open detail view
                                if let Some(&id) = self.filtered_ids.get(self.list_cursor) {
                                    self.detail_entity_id = Some(id);
                                    self.detail_scroll = 0;
                                    self.view_stack.push(self.sub_view);
                                    self.sub_view = SubView::Detail;
                                }
                            } else {
                                // Single click: select entity
                                self.list_cursor = target_idx;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        match self.sub_view {
            SubView::List => draw_entity_list(frame, self, area),
            SubView::Detail => draw_entity_detail(frame, self, area),
        }
    }

    fn status_hint(&self) -> &str {
        match self.explorer_input {
            ExplorerInput::Search => "Enter:confirm  Esc:cancel",
            ExplorerInput::Normal => match self.sub_view {
                SubView::List => "j/k:navigate  Enter:select  /:search  Tab:view  ?:help  q:quit",
                SubView::Detail => "j/k:scroll  Esc:back  ?:help  q:quit",
            },
        }
    }
}

/// Draw the entity list view.
fn draw_entity_list(frame: &mut Frame, tab: &ExplorerTab, area: Rect) {
    let items: Vec<ListItem> = tab
        .filtered_ids
        .iter()
        .map(|id| {
            let entity = tab.world.get_entity(*id);
            match entity {
                Some(e) => {
                    let kind_str = if let Some(subtype) = e.location_subtype() {
                        format!("{} ({})", e.kind, subtype)
                    } else {
                        e.kind.to_string()
                    };
                    let line = Line::from(vec![
                        Span::styled(&e.name, Style::default().fg(Color::White).bold()),
                        Span::raw("  "),
                        Span::styled(kind_str, Style::default().fg(Color::DarkGray)),
                    ]);
                    ListItem::new(line)
                }
                None => ListItem::new("???"),
            }
        })
        .collect();

    let title = if tab.search_query.is_empty() {
        format!(" Entities ({}) ", tab.filtered_ids.len())
    } else {
        format!(
            " Entities ({}) \u{2014} filter: \"{}\" ",
            tab.filtered_ids.len(),
            tab.search_query
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold())
        .highlight_symbol("\u{25b6} ");

    let mut state = ListState::default();
    state.select(Some(tab.list_cursor));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Draw the entity detail view.
fn draw_entity_detail(frame: &mut Frame, tab: &ExplorerTab, area: Rect) {
    let entity = match tab.detail_entity_id.and_then(|id| tab.world.get_entity(id)) {
        Some(e) => e,
        None => {
            let msg = Paragraph::new("No entity selected")
                .block(Block::default().title(" Detail ").borders(Borders::ALL));
            frame.render_widget(msg, area);
            return;
        }
    };

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Name and kind header
    let kind_str = if let Some(subtype) = entity.location_subtype() {
        format!("{} ({})", entity.kind, subtype)
    } else {
        entity.kind.to_string()
    };

    lines.push(Line::from(vec![
        Span::styled(entity.name.clone(), Style::default().fg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::styled(
            format!("[{kind_str}]"),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(""));

    // Description
    if !entity.description.is_empty() {
        for desc_line in entity.description.lines() {
            lines.push(Line::from(Span::styled(
                desc_line.trim().to_string(),
                Style::default().fg(Color::White),
            )));
        }
        lines.push(Line::from(""));
    }

    // Component-specific fields
    if let Some(char_comp) = &entity.components.character {
        lines.push(Line::from(Span::styled(
            "Character",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref species) = char_comp.species {
            lines.push(field_line("  species", species));
        }
        if let Some(ref occupation) = char_comp.occupation {
            lines.push(field_line("  occupation", occupation));
        }
        lines.push(field_line("  status", &format!("{:?}", char_comp.status)));
        if !char_comp.traits.is_empty() {
            lines.push(field_line("  traits", &char_comp.traits.join(", ")));
        }
        lines.push(Line::from(""));
    }

    if let Some(loc_comp) = &entity.components.location {
        lines.push(Line::from(Span::styled(
            "Location",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if loc_comp.location_type != "location" {
            lines.push(field_line("  type", &loc_comp.location_type));
        }
        if let Some(ref climate) = loc_comp.climate {
            lines.push(field_line("  climate", climate));
        }
        if let Some(ref terrain) = loc_comp.terrain {
            lines.push(field_line("  terrain", terrain));
        }
        if let Some(population) = loc_comp.population {
            lines.push(field_line("  population", &population.to_string()));
        }
        lines.push(Line::from(""));
    }

    if let Some(faction_comp) = &entity.components.faction {
        lines.push(Line::from(Span::styled(
            "Faction",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref ft) = faction_comp.faction_type {
            lines.push(field_line("  type", ft));
        }
        if !faction_comp.values.is_empty() {
            lines.push(field_line("  values", &faction_comp.values.join(", ")));
        }
        lines.push(Line::from(""));
    }

    if let Some(event_comp) = &entity.components.event {
        lines.push(Line::from(Span::styled(
            "Event",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref et) = event_comp.event_type {
            lines.push(field_line("  type", et));
        }
        if let Some(ref date) = event_comp.date {
            lines.push(field_line("  date", &date.to_string()));
        }
        if let Some(ref outcome) = event_comp.outcome {
            lines.push(field_line("  outcome", outcome));
        }
        lines.push(Line::from(""));
    }

    if let Some(item_comp) = &entity.components.item {
        lines.push(Line::from(Span::styled(
            "Item",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref it) = item_comp.item_type {
            lines.push(field_line("  type", it));
        }
        if let Some(ref rarity) = item_comp.rarity {
            lines.push(field_line("  rarity", rarity));
        }
        lines.push(Line::from(""));
    }

    if let Some(lore_comp) = &entity.components.lore {
        lines.push(Line::from(Span::styled(
            "Lore",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if let Some(ref lt) = lore_comp.lore_type {
            lines.push(field_line("  type", lt));
        }
        if let Some(ref source) = lore_comp.source {
            lines.push(field_line("  source", source));
        }
        lines.push(Line::from(""));
    }

    // Properties
    if !entity.properties.is_empty() {
        lines.push(Line::from(Span::styled(
            "Properties",
            Style::default().fg(Color::Yellow).bold(),
        )));
        let mut props: Vec<_> = entity.properties.iter().collect();
        props.sort_by_key(|(k, _)| (*k).clone());
        for (key, value) in props {
            lines.push(field_line(&format!("  {key}"), &value.to_string()));
        }
        lines.push(Line::from(""));
    }

    // Relationships
    let rels = tab.world.relationships_of(entity.id);
    if !rels.is_empty() {
        lines.push(Line::from(Span::styled(
            "Relationships",
            Style::default().fg(Color::Yellow).bold(),
        )));
        for rel in &rels {
            let other_id = if rel.source == entity.id {
                rel.target
            } else {
                rel.source
            };
            let other_name = tab.world.entity_name(other_id);
            let phrase = rel.kind.as_phrase();
            let label = if let Some(ref l) = rel.label {
                format!("{phrase} ({l}) -> {other_name}")
            } else {
                format!("{phrase} -> {other_name}")
            };
            lines.push(Line::from(vec![
                Span::styled("  ".to_string(), Style::default()),
                Span::styled(label, Style::default().fg(Color::Green)),
            ]));
        }
    }

    let title = format!(" {} ", entity.name);
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false })
        .scroll((tab.detail_scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Format a labeled field line.
fn field_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<14}"), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    use ww_core::{WorldMeta, entity::EntityKind};

    /// Create a test world with several entities for testing.
    fn create_test_world() -> World {
        let mut world = World::new(WorldMeta::new("test-world"));
        let _ = world.add_entity(Entity::new(EntityKind::Character, "Alice"));
        let _ = world.add_entity(Entity::new(EntityKind::Character, "Bob"));
        let _ = world.add_entity(Entity::new(EntityKind::Location, "Village"));
        let _ = world.add_entity(Entity::new(EntityKind::Faction, "Guild"));
        let _ = world.add_entity(Entity::new(EntityKind::Event, "Battle"));
        world
    }

    #[test]
    fn explorer_tab_initializes_with_all_entities() {
        let world = create_test_world();
        let tab = ExplorerTab::new(world);

        assert_eq!(tab.filtered_ids.len(), 5, "Should have 5 entities");
        assert_eq!(tab.list_cursor, 0, "Cursor should start at 0");
        assert_eq!(tab.sub_view, SubView::List, "Should start in list view");
        assert_eq!(
            tab.explorer_input,
            ExplorerInput::Normal,
            "Should start in normal mode"
        );
    }

    #[test]
    fn mouse_scroll_up_decrements_list_cursor() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 3;

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 2, "ScrollUp should decrement cursor");
    }

    #[test]
    fn mouse_scroll_up_at_zero_stays_zero() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 0;

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(
            tab.list_cursor, 0,
            "ScrollUp at 0 should stay at 0 (saturating_sub)"
        );
    }

    #[test]
    fn mouse_scroll_down_increments_list_cursor() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 1;

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 2, "ScrollDown should increment cursor");
    }

    #[test]
    fn mouse_scroll_down_at_end_stays_at_end() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 4; // Last entity (5 entities, 0-indexed)

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 4, "ScrollDown at end should stay at end");
    }

    #[test]
    fn mouse_scroll_up_decrements_detail_scroll() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.sub_view = SubView::Detail;
        tab.detail_scroll = 10;

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(
            tab.detail_scroll, 9,
            "ScrollUp in detail should decrement scroll"
        );
    }

    #[test]
    fn mouse_scroll_down_increments_detail_scroll() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.sub_view = SubView::Detail;
        tab.detail_scroll = 5;

        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(
            tab.detail_scroll, 6,
            "ScrollDown in detail should increment scroll"
        );
    }

    #[test]
    fn mouse_click_selects_entity_at_row() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 0;

        // Click on row 5 (row offset: 5 - 2 = 3, so index 3)
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(
            tab.list_cursor, 3,
            "Click on row 5 should select entity at index 3 (row - 2)"
        );
        assert_eq!(
            tab.sub_view,
            SubView::List,
            "Single click should stay in list view"
        );
    }

    #[test]
    fn mouse_click_on_selected_opens_detail() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 1;

        // Click on row 3 (row offset: 3 - 2 = 1, same as cursor)
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 3,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);
        assert_eq!(
            tab.sub_view,
            SubView::Detail,
            "Click on selected entity should open detail"
        );
        assert!(
            tab.detail_entity_id.is_some(),
            "Detail entity ID should be set"
        );
        assert_eq!(tab.detail_scroll, 0, "Detail scroll should be reset to 0");
    }

    #[test]
    fn mouse_click_to_detail_pushes_to_view_stack() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 2;

        assert_eq!(tab.view_stack.len(), 0, "View stack should start empty");

        // Click on row 4 (row offset: 4 - 2 = 2, same as cursor)
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 4,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };

        tab.handle_mouse(mouse);

        assert_eq!(
            tab.view_stack.len(),
            1,
            "View stack should have one entry after opening detail"
        );
        assert_eq!(
            tab.view_stack[0],
            SubView::List,
            "View stack should contain List"
        );
    }

    #[test]
    fn esc_key_from_detail_goes_back_to_list() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);

        // Set up: in detail view with view stack
        tab.view_stack.push(SubView::List);
        tab.sub_view = SubView::Detail;
        tab.detail_entity_id = Some(tab.filtered_ids[0]);

        let key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());

        tab.handle_key(key);

        assert_eq!(
            tab.sub_view,
            SubView::List,
            "Esc from detail should go back to list"
        );
        assert_eq!(
            tab.view_stack.len(),
            0,
            "View stack should be empty after going back"
        );
    }

    #[test]
    fn enter_key_opens_detail_and_pushes_to_stack() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 1;

        assert_eq!(tab.view_stack.len(), 0, "View stack should start empty");

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty());

        tab.handle_key(key);

        assert_eq!(
            tab.sub_view,
            SubView::Detail,
            "Enter should open detail view"
        );
        assert_eq!(tab.view_stack.len(), 1, "View stack should have one entry");
        assert_eq!(
            tab.view_stack[0],
            SubView::List,
            "View stack should contain List"
        );
        assert!(
            tab.detail_entity_id.is_some(),
            "Detail entity ID should be set"
        );
    }

    #[test]
    fn mouse_click_row_offset_calculation() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);

        // Test row offset calculation: row - 2
        // Row 2 is first entity (index 0), row 3 is second (index 1), etc.

        // Click row 3 -> index 1
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 3,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 1, "Row 3 should map to index 1 (row - 2)");

        // Click row 4 -> index 2
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 4,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 2, "Row 4 should map to index 2 (row - 2)");

        // Click row 6 -> index 4
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 6,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 4, "Row 6 should map to index 4 (row - 2)");

        // Click row 2 -> index 0
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 2,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);
        assert_eq!(tab.list_cursor, 0, "Row 2 should map to index 0 (row - 2)");
    }

    #[test]
    fn mouse_click_above_row_2_is_ignored() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 2;

        // Click on row 0 (tab bar) - should be ignored
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);
        assert_eq!(
            tab.list_cursor, 2,
            "Click on row 0 should be ignored, cursor unchanged"
        );

        // Click on row 1 (border/title) - should be ignored
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 1,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);
        assert_eq!(
            tab.list_cursor, 2,
            "Click on row 1 should be ignored, cursor unchanged"
        );
    }

    #[test]
    fn mouse_click_out_of_bounds_is_ignored() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);
        tab.list_cursor = 1;

        // Click on row 20 (beyond 5 entities)
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 20,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);

        assert_eq!(
            tab.list_cursor, 1,
            "Out of bounds click should not change cursor"
        );
        assert_eq!(
            tab.sub_view,
            SubView::List,
            "Out of bounds click should not change view"
        );
    }

    #[test]
    fn mouse_events_in_detail_view_do_not_select_entities() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);

        // Set up: in detail view
        tab.view_stack.push(SubView::List);
        tab.sub_view = SubView::Detail;
        tab.detail_entity_id = Some(tab.filtered_ids[2]);
        tab.list_cursor = 2;

        // Click on row 5 - should be ignored in detail view
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        tab.handle_mouse(mouse);

        assert_eq!(
            tab.sub_view,
            SubView::Detail,
            "Click in detail view should not change view"
        );
        assert_eq!(
            tab.list_cursor, 2,
            "Click in detail view should not change list cursor"
        );
    }

    #[test]
    fn keyboard_navigation_works() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);

        // Test j key (down)
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('j'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(tab.list_cursor, 1, "j should move down");

        // Test k key (up)
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('k'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(tab.list_cursor, 0, "k should move up");

        // Test g key (top)
        tab.list_cursor = 3;
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('g'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(tab.list_cursor, 0, "g should move to top");

        // Test G key (bottom)
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('G'),
            crossterm::event::KeyModifiers::SHIFT,
        ));
        assert_eq!(tab.list_cursor, 4, "G should move to bottom");
    }

    #[test]
    fn search_mode_filters_entities() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);

        // Enter search mode
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(
            tab.explorer_input,
            ExplorerInput::Search,
            "/ should enter search mode"
        );

        // Type search query
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        ));
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('l'),
            crossterm::event::KeyModifiers::empty(),
        ));

        assert_eq!(tab.search_query, "al", "Search query should be 'al'");

        // Confirm search
        tab.handle_key(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        ));

        assert_eq!(
            tab.explorer_input,
            ExplorerInput::Normal,
            "Enter should exit search mode"
        );
        assert!(
            tab.filtered_ids.len() < 5,
            "Filtered list should be smaller after search"
        );
    }

    #[test]
    fn search_mode_esc_cancels() {
        let world = create_test_world();
        let mut tab = ExplorerTab::new(world);

        // Enter search mode and type
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::empty(),
        ));
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('x'),
            crossterm::event::KeyModifiers::empty(),
        ));
        tab.handle_key(KeyEvent::new(
            KeyCode::Char('y'),
            crossterm::event::KeyModifiers::empty(),
        ));

        // Cancel with Esc
        tab.handle_key(KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ));

        assert_eq!(
            tab.explorer_input,
            ExplorerInput::Normal,
            "Esc should exit search mode"
        );
        assert_eq!(tab.search_query, "", "Esc should clear search query");
        assert_eq!(
            tab.filtered_ids.len(),
            5,
            "Filtered list should be restored"
        );
    }
}
