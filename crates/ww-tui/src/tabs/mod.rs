//! Tab definitions, trait, and tab bar rendering.

pub mod dice;
pub mod explorer;
pub mod graph;
pub mod play;
pub mod sheet;
pub mod solo;
pub mod timeline;

use ratatui::prelude::*;

/// Identifies which tab is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    /// Entity list and detail explorer.
    Explorer,
    /// Relationship graph view.
    Graph,
    /// Chronological timeline view.
    Timeline,
    /// Interactive fiction play session.
    Play,
    /// Solo TTRPG session.
    Solo,
    /// Character sheet viewer.
    Sheet,
    /// Dice roller.
    Dice,
}

impl TabId {
    /// All tab IDs in display order.
    pub const ALL: [TabId; 7] = [
        TabId::Explorer,
        TabId::Graph,
        TabId::Timeline,
        TabId::Play,
        TabId::Solo,
        TabId::Sheet,
        TabId::Dice,
    ];

    /// Parse a tab name from a string.
    pub fn from_name(name: &str) -> Option<TabId> {
        match name.to_lowercase().as_str() {
            "explorer" | "entities" => Some(TabId::Explorer),
            "graph" => Some(TabId::Graph),
            "timeline" => Some(TabId::Timeline),
            "play" => Some(TabId::Play),
            "solo" => Some(TabId::Solo),
            "sheet" => Some(TabId::Sheet),
            "dice" => Some(TabId::Dice),
            _ => None,
        }
    }

    /// Index of this tab in the tab bar.
    pub fn index(self) -> usize {
        TabId::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    /// Get the next tab (wrapping).
    pub fn next(self) -> TabId {
        let idx = (self.index() + 1) % TabId::ALL.len();
        TabId::ALL[idx]
    }

    /// Get the previous tab (wrapping).
    pub fn prev(self) -> TabId {
        let idx = if self.index() == 0 {
            TabId::ALL.len() - 1
        } else {
            self.index() - 1
        };
        TabId::ALL[idx]
    }
}

/// Whether a tab consumes keyboard input or uses vim-like navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Vim-like navigation: hjkl, /, Tab, number keys. Top-level handles tab switching.
    VimNav,
    /// Text input: the tab has its own input field. Most keys go to the tab.
    TextInput,
}

/// Trait that each tab screen implements.
pub trait Tab {
    /// Return the input mode for event routing.
    fn input_mode(&self) -> InputMode;

    /// Handle a key event. Return `true` if the app should quit.
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool;

    /// Handle a mouse event.
    fn handle_mouse(&mut self, _mouse: crossterm::event::MouseEvent) {}

    /// Draw the tab content into the given area.
    fn draw(&self, frame: &mut Frame, area: Rect);

    /// Return context-sensitive status bar text.
    fn status_hint(&self) -> &str;
}

/// Draw the tab bar.
pub fn draw_tab_bar(frame: &mut Frame, active: TabId, area: Rect) {
    let titles = [
        "[1]Explorer",
        "[2]Graph",
        "[3]Timeline",
        "[4]Play",
        "[5]Solo",
        "[6]Sheet",
        "[7]Dice",
    ];

    // Create spans with proper styling for each tab
    let active_idx = active.index();
    let mut spans = Vec::new();

    for (i, title) in titles.iter().enumerate() {
        // Add divider before this tab (except for first)
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        }

        // Add the tab title with appropriate styling
        let style = if i == active_idx {
            Style::default().fg(Color::White).bold()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(*title, style));
    }

    let line = Line::from(spans);
    let paragraph = ratatui::widgets::Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
