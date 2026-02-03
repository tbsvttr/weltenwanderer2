use ww_core::entity::{Entity, EntityId, EntityKind};
use ww_core::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    EntityList,
    EntityDetail,
    Graph,
    Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    pub world: World,
    pub active_view: ActiveView,
    pub input_mode: InputMode,
    pub show_help: bool,

    // Entity list state
    pub list_cursor: usize,
    pub list_filter: Option<EntityKind>,
    pub search_query: String,
    pub filtered_ids: Vec<EntityId>,

    // Entity detail state
    pub detail_entity_id: Option<EntityId>,
    pub detail_scroll: u16,

    // Graph state
    pub graph_scroll: u16,

    // Timeline state
    pub timeline_cursor: usize,

    // Navigation stack (for back)
    pub view_stack: Vec<ActiveView>,
}

impl App {
    pub fn new(world: World) -> Self {
        let mut app = Self {
            world,
            active_view: ActiveView::EntityList,
            input_mode: InputMode::Normal,
            show_help: false,
            list_cursor: 0,
            list_filter: None,
            search_query: String::new(),
            filtered_ids: Vec::new(),
            detail_entity_id: None,
            detail_scroll: 0,
            graph_scroll: 0,
            timeline_cursor: 0,
            view_stack: Vec::new(),
        };
        app.update_filtered_list();
        app
    }

    pub fn update_filtered_list(&mut self) {
        let mut query = self.world.query();

        if let Some(ref kind) = self.list_filter {
            query = query.kind(kind.clone());
        }

        if !self.search_query.is_empty() {
            query = query.name_contains(&self.search_query);
        }

        self.filtered_ids = query.execute().iter().map(|e| e.id).collect();
        // Clamp cursor
        if self.list_cursor >= self.filtered_ids.len() && !self.filtered_ids.is_empty() {
            self.list_cursor = self.filtered_ids.len() - 1;
        }
    }

    pub fn selected_entity(&self) -> Option<&Entity> {
        self.filtered_ids
            .get(self.list_cursor)
            .and_then(|id| self.world.get_entity(*id))
    }

    // Navigation
    pub fn move_down(&mut self) {
        match self.active_view {
            ActiveView::EntityList => {
                if self.list_cursor + 1 < self.filtered_ids.len() {
                    self.list_cursor += 1;
                }
            }
            ActiveView::EntityDetail => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            ActiveView::Graph => {
                self.graph_scroll = self.graph_scroll.saturating_add(1);
            }
            ActiveView::Timeline => {
                let count = self.timeline_entry_count();
                if self.timeline_cursor + 1 < count {
                    self.timeline_cursor += 1;
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        match self.active_view {
            ActiveView::EntityList => {
                self.list_cursor = self.list_cursor.saturating_sub(1);
            }
            ActiveView::EntityDetail => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            ActiveView::Graph => {
                self.graph_scroll = self.graph_scroll.saturating_sub(1);
            }
            ActiveView::Timeline => {
                self.timeline_cursor = self.timeline_cursor.saturating_sub(1);
            }
        }
    }

    pub fn move_to_top(&mut self) {
        match self.active_view {
            ActiveView::EntityList => self.list_cursor = 0,
            ActiveView::EntityDetail => self.detail_scroll = 0,
            ActiveView::Graph => self.graph_scroll = 0,
            ActiveView::Timeline => self.timeline_cursor = 0,
        }
    }

    pub fn move_to_bottom(&mut self) {
        match self.active_view {
            ActiveView::EntityList => {
                if !self.filtered_ids.is_empty() {
                    self.list_cursor = self.filtered_ids.len() - 1;
                }
            }
            ActiveView::EntityDetail | ActiveView::Graph => {
                // Scroll to bottom handled in draw
            }
            ActiveView::Timeline => {
                let count = self.timeline_entry_count();
                if count > 0 {
                    self.timeline_cursor = count - 1;
                }
            }
        }
    }

    pub fn select(&mut self) {
        if self.active_view == ActiveView::EntityList {
            if let Some(entity) = self.selected_entity() {
                self.detail_entity_id = Some(entity.id);
                self.detail_scroll = 0;
                self.view_stack.push(self.active_view);
                self.active_view = ActiveView::EntityDetail;
            }
        } else if self.active_view == ActiveView::Timeline {
            // Select event from timeline → detail
            let timeline = ww_core::timeline::Timeline::from_world(&self.world);
            if let Some(entry) = timeline.entries().get(self.timeline_cursor) {
                self.detail_entity_id = Some(entry.entity.id);
                self.detail_scroll = 0;
                self.view_stack.push(self.active_view);
                self.active_view = ActiveView::EntityDetail;
            }
        }
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.view_stack.pop() {
            self.active_view = prev;
        }
    }

    pub fn next_view(&mut self) {
        self.active_view = match self.active_view {
            ActiveView::EntityList => ActiveView::Graph,
            ActiveView::Graph => ActiveView::Timeline,
            ActiveView::Timeline => ActiveView::EntityList,
            ActiveView::EntityDetail => ActiveView::EntityList,
        };
        self.view_stack.clear();
    }

    pub fn switch_view(&mut self, view: ActiveView) {
        self.active_view = view;
        self.view_stack.clear();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    // Search
    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_query.clear();
    }

    pub fn cancel_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_query.clear();
        self.update_filtered_list();
    }

    pub fn confirm_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.list_cursor = 0;
        self.update_filtered_list();
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.list_cursor = 0;
        self.update_filtered_list();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.list_cursor = 0;
        self.update_filtered_list();
    }

    fn timeline_entry_count(&self) -> usize {
        ww_core::timeline::Timeline::from_world(&self.world).len()
    }
}
