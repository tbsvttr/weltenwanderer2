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
    // First, clear the entire area with a black background block
    let clear_block = ratatui::widgets::Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, area);

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
            spans.push(Span::styled(
                " | ",
                Style::default().fg(Color::DarkGray).bg(Color::Black),
            ));
        }

        // Add the tab title with appropriate styling
        let style = if i == active_idx {
            Style::default().fg(Color::White).bg(Color::Black).bold()
        } else {
            Style::default().fg(Color::DarkGray).bg(Color::Black)
        };
        spans.push(Span::styled(*title, style));
    }

    // Calculate the total length of the tab bar text
    let text_len: usize = spans.iter().map(|s| s.content.len()).sum();

    // Pad with spaces to fill the entire width and prevent text bleed-through
    let padding_len = (area.width as usize).saturating_sub(text_len);
    if padding_len > 0 {
        spans.push(Span::styled(
            " ".repeat(padding_len),
            Style::default().fg(Color::Black).bg(Color::Black),
        ));
    }

    let line = Line::from(spans);
    let paragraph = ratatui::widgets::Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod rendering_tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn tab_bar_renders_all_tabs() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Explorer, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let line = buffer.content()[0..100]
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        // Verify all tab labels are present
        assert!(line.contains("[1]Explorer"), "Explorer tab missing");
        assert!(line.contains("[2]Graph"), "Graph tab missing");
        assert!(line.contains("[3]Timeline"), "Timeline tab missing");
        assert!(line.contains("[4]Play"), "Play tab missing");
        assert!(line.contains("[5]Solo"), "Solo tab missing");
        assert!(line.contains("[6]Sheet"), "Sheet tab missing");
        assert!(line.contains("[7]Dice"), "Dice tab missing");
    }

    #[test]
    fn tab_bar_renders_dividers() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Explorer, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let line = buffer.content()[0..100]
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        // Verify dividers are present between tabs
        let divider_count = line.matches(" | ").count();
        assert_eq!(divider_count, 6, "Should have 6 dividers between 7 tabs");
    }

    #[test]
    fn tab_bar_renders_in_correct_order() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Explorer, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let line = buffer.content()[0..100]
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        // Find positions of each tab
        let explorer_pos = line.find("[1]Explorer").unwrap();
        let graph_pos = line.find("[2]Graph").unwrap();
        let timeline_pos = line.find("[3]Timeline").unwrap();
        let play_pos = line.find("[4]Play").unwrap();
        let solo_pos = line.find("[5]Solo").unwrap();
        let sheet_pos = line.find("[6]Sheet").unwrap();
        let dice_pos = line.find("[7]Dice").unwrap();

        // Verify order
        assert!(
            explorer_pos < graph_pos,
            "Explorer should come before Graph"
        );
        assert!(
            graph_pos < timeline_pos,
            "Graph should come before Timeline"
        );
        assert!(timeline_pos < play_pos, "Timeline should come before Play");
        assert!(play_pos < solo_pos, "Play should come before Solo");
        assert!(solo_pos < sheet_pos, "Solo should come before Sheet");
        assert!(sheet_pos < dice_pos, "Sheet should come before Dice");
    }

    #[test]
    fn tab_bar_active_tab_has_different_style() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Graph, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        // Find the Graph tab cells
        let line_str = buffer.content()[0..100]
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        let graph_start = line_str.find("[2]Graph").unwrap();

        // Check that the Graph tab has different styling (white/bold vs dark gray)
        let graph_cell = &buffer.content()[graph_start + 1]; // Check the '2' character
        assert_eq!(
            graph_cell.fg,
            Color::White,
            "Active tab should be white, not dark gray"
        );
    }

    #[test]
    fn tab_bar_fills_entire_area() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Explorer, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        // Verify all cells in the tab bar row have black background
        for i in 0..100 {
            let cell = &buffer.content()[i];
            assert_eq!(
                cell.bg,
                Color::Black,
                "Cell {i} should have black background to prevent bleed-through"
            );
        }
    }

    #[test]
    fn tab_bar_exact_positions_match_hit_testing() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Explorer, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let line = buffer.content()[0..100]
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        // Verify the exact layout matches what our hit-testing expects
        // Explorer: [0, 14)
        assert_eq!(&line[0..11], "[1]Explorer", "Explorer position mismatch");
        assert_eq!(&line[11..14], " | ", "First divider position mismatch");

        // Graph: [14, 25)
        assert_eq!(&line[14..22], "[2]Graph", "Graph position mismatch");
        assert_eq!(&line[22..25], " | ", "Second divider position mismatch");

        // Timeline: [25, 39)
        assert_eq!(&line[25..36], "[3]Timeline", "Timeline position mismatch");
        assert_eq!(&line[36..39], " | ", "Third divider position mismatch");

        // Play: [39, 49)
        assert_eq!(&line[39..46], "[4]Play", "Play position mismatch");
        assert_eq!(&line[46..49], " | ", "Fourth divider position mismatch");

        // Solo: [49, 59)
        assert_eq!(&line[49..56], "[5]Solo", "Solo position mismatch");
        assert_eq!(&line[56..59], " | ", "Fifth divider position mismatch");

        // Sheet: [59, 70)
        assert_eq!(&line[59..67], "[6]Sheet", "Sheet position mismatch");
        assert_eq!(&line[67..70], " | ", "Sixth divider position mismatch");

        // Dice: [70, 77)
        assert_eq!(&line[70..77], "[7]Dice", "Dice position mismatch");
    }

    #[test]
    fn tab_bar_no_text_overlap() {
        let backend = TestBackend::new(100, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 1);
                draw_tab_bar(frame, TabId::Timeline, area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let line = buffer.content()[0..100]
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        // Verify no unexpected characters between known positions
        // Everything should be either tab labels, dividers, or spaces
        let expected_chars = "[1234567]ExplorerGaphTimelinPaySoDceSht |";
        for (i, c) in line.chars().enumerate() {
            if i < 77 && !c.is_whitespace() {
                // Character should be part of a tab label or divider
                let is_valid = expected_chars.contains(c);
                assert!(
                    is_valid,
                    "Unexpected character '{c}' at position {i} (possible text bleed-through)"
                );
            }
        }
    }
}
