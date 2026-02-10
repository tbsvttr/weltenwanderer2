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
        use crossterm::event::MouseEventKind;

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
