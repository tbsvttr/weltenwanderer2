//! Character sheet viewer tab.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap};

use ww_core::World;
use ww_core::entity::{EntityId, EntityKind};

use super::{InputMode, Tab};

/// Sub-view within the sheet tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubView {
    /// Character selection list.
    List,
    /// Character sheet detail.
    Detail,
}

/// Character sheet viewer tab state.
pub struct SheetTab {
    /// The world data.
    world: World,
    /// Character entity IDs.
    character_ids: Vec<EntityId>,
    /// Cursor in the character list.
    list_cursor: usize,
    /// Currently selected character for detail view.
    selected: Option<EntityId>,
    /// Scroll offset in the detail view.
    detail_scroll: u16,
    /// Current sub-view.
    sub_view: SubView,
}

impl SheetTab {
    /// Create a new sheet tab for the given world.
    pub fn new(world: World) -> Self {
        let character_ids: Vec<EntityId> = world
            .query()
            .kind(EntityKind::Character)
            .execute()
            .iter()
            .map(|e| e.id)
            .collect();

        Self {
            world,
            character_ids,
            list_cursor: 0,
            selected: None,
            detail_scroll: 0,
            sub_view: SubView::List,
        }
    }
}

impl Tab for SheetTab {
    fn input_mode(&self) -> InputMode {
        InputMode::VimNav
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.sub_view {
            SubView::List => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.list_cursor + 1 < self.character_ids.len() {
                        self.list_cursor += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.list_cursor = self.list_cursor.saturating_sub(1);
                }
                KeyCode::Char('g') => self.list_cursor = 0,
                KeyCode::Char('G') => {
                    if !self.character_ids.is_empty() {
                        self.list_cursor = self.character_ids.len() - 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(&id) = self.character_ids.get(self.list_cursor) {
                        self.selected = Some(id);
                        self.detail_scroll = 0;
                        self.sub_view = SubView::Detail;
                    }
                }
                _ => {}
            },
            SubView::Detail => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.detail_scroll = self.detail_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
                KeyCode::Char('g') => self.detail_scroll = 0,
                KeyCode::Esc => {
                    self.sub_view = SubView::List;
                    self.selected = None;
                }
                _ => {}
            },
        }
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => match self.sub_view {
                SubView::List => self.list_cursor = self.list_cursor.saturating_sub(1),
                SubView::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
            },
            MouseEventKind::ScrollDown => match self.sub_view {
                SubView::List => {
                    if self.list_cursor + 1 < self.character_ids.len() {
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
            SubView::List => draw_character_list(frame, self, area),
            SubView::Detail => draw_sheet_detail(frame, self, area),
        }
    }

    fn status_hint(&self) -> &str {
        match self.sub_view {
            SubView::List => "j/k:navigate  Enter:select  Tab:view  ?:help  q:quit",
            SubView::Detail => "j/k:scroll  Esc:back  ?:help  q:quit",
        }
    }
}

/// Draw the character selection list.
fn draw_character_list(frame: &mut Frame, tab: &SheetTab, area: Rect) {
    let items: Vec<ListItem> = tab
        .character_ids
        .iter()
        .map(|id| {
            let name = tab.world.entity_name(*id);
            ListItem::new(Line::from(Span::styled(
                name.to_string(),
                Style::default().fg(Color::White).bold(),
            )))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Characters ({}) ", tab.character_ids.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold())
        .highlight_symbol("\u{25b6} ");

    let mut state = ListState::default();
    state.select(Some(tab.list_cursor));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Draw the character sheet detail view.
fn draw_sheet_detail(frame: &mut Frame, tab: &SheetTab, area: Rect) {
    let entity_id = match tab.selected {
        Some(id) => id,
        None => {
            let msg = Paragraph::new("No character selected")
                .block(Block::default().title(" Sheet ").borders(Borders::ALL));
            frame.render_widget(msg, area);
            return;
        }
    };

    let entity = match tab.world.get_entity(entity_id) {
        Some(e) => e,
        None => {
            let msg = Paragraph::new("Entity not found")
                .block(Block::default().title(" Sheet ").borders(Borders::ALL));
            frame.render_widget(msg, area);
            return;
        }
    };

    let ruleset = match ww_mechanics::RuleSet::from_world(&tab.world) {
        Ok(rs) => rs,
        Err(_) => {
            let msg = Paragraph::new(Span::styled(
                "No ruleset defined in this world",
                Style::default().fg(Color::DarkGray),
            ))
            .block(
                Block::default()
                    .title(format!(" {} ", entity.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
            frame.render_widget(msg, area);
            return;
        }
    };

    let sheet = match ww_mechanics::CharacterSheet::from_entity(entity, &ruleset) {
        Ok(s) => s,
        Err(e) => {
            let msg = Paragraph::new(Span::styled(
                format!("Error: {e}"),
                Style::default().fg(Color::Red),
            ))
            .block(
                Block::default()
                    .title(format!(" {} ", entity.name))
                    .borders(Borders::ALL),
            );
            frame.render_widget(msg, area);
            return;
        }
    };

    // Build the sheet as lines
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Attributes
    if !sheet.attributes.is_empty() {
        lines.push(Line::from(Span::styled(
            "Attributes",
            Style::default().fg(Color::Yellow).bold(),
        )));
        let mut attrs: Vec<_> = sheet.attributes.iter().collect();
        attrs.sort_by_key(|(k, _)| k.to_lowercase());
        let mut col = 0;
        let mut row_spans: Vec<Span<'static>> = Vec::new();
        for (name, value) in &attrs {
            let text = format!("  {name}: {value}");
            row_spans.push(Span::styled(
                format!("{text:<20}"),
                Style::default().fg(Color::White),
            ));
            col += 1;
            if col >= 2 {
                lines.push(Line::from(std::mem::take(&mut row_spans)));
                col = 0;
            }
        }
        if !row_spans.is_empty() {
            lines.push(Line::from(row_spans));
        }
        lines.push(Line::from(""));
    }

    // Skills
    if !sheet.skills.is_empty() {
        lines.push(Line::from(Span::styled(
            "Skills",
            Style::default().fg(Color::Yellow).bold(),
        )));
        let mut skills: Vec<_> = sheet.skills.iter().collect();
        skills.sort_by_key(|(k, _)| k.to_lowercase());
        let mut col = 0;
        let mut row_spans: Vec<Span<'static>> = Vec::new();
        for (name, value) in &skills {
            let text = format!("  {name}: {value}");
            row_spans.push(Span::styled(
                format!("{text:<20}"),
                Style::default().fg(Color::White),
            ));
            col += 1;
            if col >= 2 {
                lines.push(Line::from(std::mem::take(&mut row_spans)));
                col = 0;
            }
        }
        if !row_spans.is_empty() {
            lines.push(Line::from(row_spans));
        }
        lines.push(Line::from(""));
    }

    // Focuses
    if !sheet.focuses.is_empty() {
        lines.push(Line::from(Span::styled(
            "Focuses",
            Style::default().fg(Color::Yellow).bold(),
        )));
        for focus in &sheet.focuses {
            lines.push(Line::from(Span::styled(
                format!("  {focus}"),
                Style::default().fg(Color::Magenta),
            )));
        }
        lines.push(Line::from(""));
    }

    // Build the text portion as a paragraph
    let text_paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((tab.detail_scroll, 0));

    // We'll split the area: text at top, tracks at bottom
    let mut tracks: Vec<_> = sheet.tracks.values().collect();
    tracks.sort_by(|a, b| a.name.cmp(&b.name));
    let track_height = tracks.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(track_height.max(1))])
        .split(
            Block::default()
                .title(format!(" {} ", sheet.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .inner(area),
        );

    // Render the outer block
    let outer_block = Block::default()
        .title(format!(" {} ", sheet.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(outer_block, area);

    // Text content
    frame.render_widget(text_paragraph, chunks[0]);

    // Tracks as gauges
    for (i, track) in tracks.iter().enumerate() {
        if i as u16 >= chunks[1].height {
            break;
        }
        let row = Rect::new(chunks[1].x, chunks[1].y + i as u16, chunks[1].width, 1);
        let fraction = track.fraction();
        let color = if fraction < 0.3 {
            Color::Red
        } else if fraction < 0.6 {
            Color::Yellow
        } else {
            Color::Green
        };
        let label = format!("{} {}/{}", track.name, track.current, track.max);
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
            .ratio(fraction.clamp(0.0, 1.0))
            .label(Span::styled(label, Style::default().fg(Color::White)));
        frame.render_widget(gauge, row);
    }
}
