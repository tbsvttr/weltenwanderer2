//! Terminal setup, teardown, and main event loop.

use std::io;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::TuiApp;
use crate::tabs::{self, InputMode, TabId};

/// Launch the TUI application.
pub fn run(mut app: TuiApp) -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("terminal error: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("terminal error: {e}"))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("terminal error: {e}"))?;

    // Ensure lazy tabs are ready for the initial tab
    let _ = app.active_tab_mut();

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    result
}

/// Main event loop.
fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| draw(frame, app))
            .map_err(|e| format!("draw error: {e}"))?;

        if app.should_quit {
            return Ok(());
        }

        let event = event::read().map_err(|e| format!("event error: {e}"))?;
        handle_event(app, event);
    }
}

/// Handle a crossterm event.
fn handle_event(app: &mut TuiApp, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        _ => {}
    }
}

/// Handle keyboard input with mode-aware tab switching.
fn handle_key(app: &mut TuiApp, key: crossterm::event::KeyEvent) {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // Ctrl+number switches tabs from any mode
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && let Some(tab) = ctrl_number_to_tab(key.code)
    {
        app.switch_tab(tab);
        return;
    }

    let mode = app.active_input_mode();

    match mode {
        InputMode::VimNav => {
            // Global keys for VimNav tabs
            match key.code {
                KeyCode::Char('q') => {
                    app.should_quit = true;
                    return;
                }
                KeyCode::Char('?') => {
                    app.show_help = !app.show_help;
                    return;
                }
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app.switch_tab(app.active_tab.prev());
                    } else {
                        app.switch_tab(app.active_tab.next());
                    }
                    return;
                }
                KeyCode::BackTab => {
                    app.switch_tab(app.active_tab.prev());
                    return;
                }
                _ => {}
            }
            // Number keys 1-7 switch tabs
            if let KeyCode::Char(c) = key.code
                && let Some(idx) = c.to_digit(10)
                && (1..=7).contains(&idx)
            {
                app.switch_tab(TabId::ALL[idx as usize - 1]);
                return;
            }
            // Forward to active tab
            if app.active_tab_mut().handle_key(key) {
                app.should_quit = true;
            }
        }
        InputMode::TextInput => {
            // In text-input mode, only Ctrl+N and Esc toggle help
            if key.code == KeyCode::Char('?')
                && app.show_help
                && !key.modifiers.contains(KeyModifiers::CONTROL)
            {
                app.show_help = false;
                return;
            }
            // Forward to active tab
            if app.active_tab_mut().handle_key(key) {
                app.should_quit = true;
            }
        }
    }
}

/// Map Ctrl+digit to a tab.
fn ctrl_number_to_tab(code: KeyCode) -> Option<TabId> {
    match code {
        KeyCode::Char('1') => Some(TabId::Explorer),
        KeyCode::Char('2') => Some(TabId::Graph),
        KeyCode::Char('3') => Some(TabId::Timeline),
        KeyCode::Char('4') => Some(TabId::Play),
        KeyCode::Char('5') => Some(TabId::Solo),
        KeyCode::Char('6') => Some(TabId::Sheet),
        KeyCode::Char('7') => Some(TabId::Dice),
        _ => None,
    }
}

/// Handle mouse events.
fn handle_mouse(app: &mut TuiApp, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if the click is on the tab bar (row 0)
            if mouse.row == 0
                && let Some(tab) = tab_bar_hit_test(mouse.column)
            {
                app.switch_tab(tab);
                return;
            }
            // Forward to active tab
            app.active_tab_mut().handle_mouse(mouse);
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            app.active_tab_mut().handle_mouse(mouse);
        }
        _ => {}
    }
}

/// Hit-test the tab bar for mouse clicks.
fn tab_bar_hit_test(col: u16) -> Option<TabId> {
    // Tab labels with dividers: "[1]Explorer | [2]Graph | [3]Timeline | [4]Play | [5]Solo | [6]Sheet | [7]Dice"
    // For better UX, make each tab's clickable area include the divider space after it.
    // This makes clicking more forgiving and prevents dead zones.

    let labels = [
        "[1]Explorer",
        "[2]Graph",
        "[3]Timeline",
        "[4]Play",
        "[5]Solo",
        "[6]Sheet",
        "[7]Dice",
    ];

    let divider_len = 3u16; // " | " is 3 chars

    let mut x = 0u16;
    for (i, label) in labels.iter().enumerate() {
        let label_len = label.len() as u16;
        // Include divider in this tab's clickable area (except for last tab)
        let clickable_width = if i < labels.len() - 1 {
            label_len + divider_len
        } else {
            label_len
        };
        let end_x = x + clickable_width;

        // Check if click is within this tab's clickable area (label + divider)
        if col >= x && col < end_x {
            return Some(TabId::ALL[i]);
        }

        // Move to next tab position
        x = end_x;
    }

    None
}

/// Main draw function.
fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Tab bar
    tabs::draw_tab_bar(frame, app.active_tab, chunks[0]);

    // Active tab content
    app.active_tab_mut().draw(frame, chunks[1]);

    // Status bar
    let hint = app.active_tab_ref().status_hint();
    let status = Paragraph::new(hint).style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(status, chunks[2]);

    // Help popup overlay
    if app.show_help {
        crate::shared::draw_help_popup(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_bar_hit_test_boundaries() {
        // Tab layout: "[1]Explorer | [2]Graph | [3]Timeline | [4]Play | [5]Solo | [6]Sheet | [7]Dice"
        // Lengths: 11, 8, 11, 7, 7, 8, 7
        // Divider: " | " (3 chars) - included in each tab's clickable area
        // Clickable areas: Tab includes label + divider (except last tab)

        // Tab 0: Explorer (cols 0-13, 11 chars + 3 divider)
        assert_eq!(tab_bar_hit_test(0), Some(TabId::Explorer));
        assert_eq!(tab_bar_hit_test(5), Some(TabId::Explorer));
        assert_eq!(tab_bar_hit_test(10), Some(TabId::Explorer));
        assert_eq!(tab_bar_hit_test(11), Some(TabId::Explorer)); // divider is part of Explorer
        assert_eq!(tab_bar_hit_test(13), Some(TabId::Explorer));

        // Tab 1: Graph (cols 14-24, 8 chars + 3 divider)
        assert_eq!(tab_bar_hit_test(14), Some(TabId::Graph));
        assert_eq!(tab_bar_hit_test(18), Some(TabId::Graph));
        assert_eq!(tab_bar_hit_test(21), Some(TabId::Graph));
        assert_eq!(tab_bar_hit_test(24), Some(TabId::Graph)); // divider is part of Graph

        // Tab 2: Timeline (cols 25-38, 11 chars + 3 divider)
        assert_eq!(tab_bar_hit_test(25), Some(TabId::Timeline));
        assert_eq!(tab_bar_hit_test(30), Some(TabId::Timeline));
        assert_eq!(tab_bar_hit_test(35), Some(TabId::Timeline));
        assert_eq!(tab_bar_hit_test(38), Some(TabId::Timeline)); // divider is part of Timeline

        // Tab 3: Play (cols 39-48, 7 chars + 3 divider)
        assert_eq!(tab_bar_hit_test(39), Some(TabId::Play));
        assert_eq!(tab_bar_hit_test(42), Some(TabId::Play));
        assert_eq!(tab_bar_hit_test(45), Some(TabId::Play));
        assert_eq!(tab_bar_hit_test(48), Some(TabId::Play)); // divider is part of Play

        // Tab 4: Solo (cols 49-58, 7 chars + 3 divider)
        assert_eq!(tab_bar_hit_test(49), Some(TabId::Solo));
        assert_eq!(tab_bar_hit_test(52), Some(TabId::Solo));
        assert_eq!(tab_bar_hit_test(55), Some(TabId::Solo));
        assert_eq!(tab_bar_hit_test(58), Some(TabId::Solo)); // divider is part of Solo

        // Tab 5: Sheet (cols 59-69, 8 chars + 3 divider)
        assert_eq!(tab_bar_hit_test(59), Some(TabId::Sheet));
        assert_eq!(tab_bar_hit_test(63), Some(TabId::Sheet));
        assert_eq!(tab_bar_hit_test(66), Some(TabId::Sheet));
        assert_eq!(tab_bar_hit_test(69), Some(TabId::Sheet)); // divider is part of Sheet

        // Tab 6: Dice (cols 70-76, 7 chars, no divider after)
        assert_eq!(tab_bar_hit_test(70), Some(TabId::Dice));
        assert_eq!(tab_bar_hit_test(73), Some(TabId::Dice));
        assert_eq!(tab_bar_hit_test(76), Some(TabId::Dice));

        // Beyond all tabs
        assert_eq!(tab_bar_hit_test(100), None);
    }

    #[test]
    fn tab_bar_hit_test_each_tab() {
        // Test clicking in the middle of each tab label
        assert_eq!(tab_bar_hit_test(5), Some(TabId::Explorer)); // middle of Explorer
        assert_eq!(tab_bar_hit_test(18), Some(TabId::Graph)); // middle of Graph
        assert_eq!(tab_bar_hit_test(30), Some(TabId::Timeline)); // middle of Timeline
        assert_eq!(tab_bar_hit_test(42), Some(TabId::Play)); // middle of Play
        assert_eq!(tab_bar_hit_test(52), Some(TabId::Solo)); // middle of Solo
        assert_eq!(tab_bar_hit_test(63), Some(TabId::Sheet)); // middle of Sheet
        assert_eq!(tab_bar_hit_test(73), Some(TabId::Dice)); // middle of Dice
    }

    #[test]
    fn tab_bar_hit_test_dividers_part_of_tabs() {
        // Test that clicking on dividers now selects the tab before them (for better UX)
        assert_eq!(tab_bar_hit_test(11), Some(TabId::Explorer)); // divider after Explorer
        assert_eq!(tab_bar_hit_test(22), Some(TabId::Graph)); // divider after Graph
        assert_eq!(tab_bar_hit_test(36), Some(TabId::Timeline)); // divider after Timeline
        assert_eq!(tab_bar_hit_test(46), Some(TabId::Play)); // divider after Play
        assert_eq!(tab_bar_hit_test(56), Some(TabId::Solo)); // divider after Solo
        assert_eq!(tab_bar_hit_test(67), Some(TabId::Sheet)); // divider after Sheet
    }

    #[test]
    fn tab_bar_lengths_match_actual_strings() {
        // Verify our assumptions about string lengths
        assert_eq!("[1]Explorer".len(), 11);
        assert_eq!("[2]Graph".len(), 8);
        assert_eq!("[3]Timeline".len(), 11);
        assert_eq!("[4]Play".len(), 7);
        assert_eq!("[5]Solo".len(), 7);
        assert_eq!("[6]Sheet".len(), 8);
        assert_eq!("[7]Dice".len(), 7);
    }

    #[test]
    fn tab_bar_hit_test_all_first_chars() {
        // Test clicking on the first character of each tab (the '[')
        assert_eq!(tab_bar_hit_test(0), Some(TabId::Explorer)); // '[' of Explorer
        assert_eq!(tab_bar_hit_test(14), Some(TabId::Graph)); // '[' of Graph
        assert_eq!(tab_bar_hit_test(25), Some(TabId::Timeline)); // '[' of Timeline
        assert_eq!(tab_bar_hit_test(39), Some(TabId::Play)); // '[' of Play
        assert_eq!(tab_bar_hit_test(49), Some(TabId::Solo)); // '[' of Solo
        assert_eq!(tab_bar_hit_test(59), Some(TabId::Sheet)); // '[' of Sheet
        assert_eq!(tab_bar_hit_test(70), Some(TabId::Dice)); // '[' of Dice
    }

    #[test]
    fn tab_bar_hit_test_all_last_chars() {
        // Test clicking on the last character of each tab label (before divider)
        assert_eq!(tab_bar_hit_test(10), Some(TabId::Explorer)); // 'r' of Explorer
        assert_eq!(tab_bar_hit_test(21), Some(TabId::Graph)); // 'h' of Graph
        assert_eq!(tab_bar_hit_test(35), Some(TabId::Timeline)); // 'e' of Timeline
        assert_eq!(tab_bar_hit_test(45), Some(TabId::Play)); // 'y' of Play
        assert_eq!(tab_bar_hit_test(55), Some(TabId::Solo)); // 'o' of Solo
        assert_eq!(tab_bar_hit_test(66), Some(TabId::Sheet)); // 't' of Sheet
        assert_eq!(tab_bar_hit_test(76), Some(TabId::Dice)); // 'e' of Dice
    }

    #[test]
    fn tab_bar_hit_test_exact_boundaries() {
        // Test the exact start and end boundaries of each tab's clickable area
        // Explorer: [0, 14)
        assert_eq!(tab_bar_hit_test(0), Some(TabId::Explorer));
        assert_eq!(tab_bar_hit_test(13), Some(TabId::Explorer));
        assert_eq!(tab_bar_hit_test(14), Some(TabId::Graph)); // boundary

        // Graph: [14, 25)
        assert_eq!(tab_bar_hit_test(14), Some(TabId::Graph));
        assert_eq!(tab_bar_hit_test(24), Some(TabId::Graph));
        assert_eq!(tab_bar_hit_test(25), Some(TabId::Timeline)); // boundary

        // Timeline: [25, 39)
        assert_eq!(tab_bar_hit_test(25), Some(TabId::Timeline));
        assert_eq!(tab_bar_hit_test(38), Some(TabId::Timeline));
        assert_eq!(tab_bar_hit_test(39), Some(TabId::Play)); // boundary

        // Play: [39, 49)
        assert_eq!(tab_bar_hit_test(39), Some(TabId::Play));
        assert_eq!(tab_bar_hit_test(48), Some(TabId::Play));
        assert_eq!(tab_bar_hit_test(49), Some(TabId::Solo)); // boundary

        // Solo: [49, 59)
        assert_eq!(tab_bar_hit_test(49), Some(TabId::Solo));
        assert_eq!(tab_bar_hit_test(58), Some(TabId::Solo));
        assert_eq!(tab_bar_hit_test(59), Some(TabId::Sheet)); // boundary

        // Sheet: [59, 70)
        assert_eq!(tab_bar_hit_test(59), Some(TabId::Sheet));
        assert_eq!(tab_bar_hit_test(69), Some(TabId::Sheet));
        assert_eq!(tab_bar_hit_test(70), Some(TabId::Dice)); // boundary

        // Dice: [70, 77)
        assert_eq!(tab_bar_hit_test(70), Some(TabId::Dice));
        assert_eq!(tab_bar_hit_test(76), Some(TabId::Dice));
        assert_eq!(tab_bar_hit_test(77), None); // beyond
    }

    #[test]
    fn tab_bar_hit_test_number_positions() {
        // Test clicking on the number digit of each tab
        assert_eq!(tab_bar_hit_test(1), Some(TabId::Explorer)); // '1'
        assert_eq!(tab_bar_hit_test(15), Some(TabId::Graph)); // '2'
        assert_eq!(tab_bar_hit_test(26), Some(TabId::Timeline)); // '3'
        assert_eq!(tab_bar_hit_test(40), Some(TabId::Play)); // '4'
        assert_eq!(tab_bar_hit_test(50), Some(TabId::Solo)); // '5'
        assert_eq!(tab_bar_hit_test(60), Some(TabId::Sheet)); // '6'
        assert_eq!(tab_bar_hit_test(71), Some(TabId::Dice)); // '7'
    }

    #[test]
    fn tab_bar_hit_test_no_gaps() {
        // Verify there are no gaps - every column from 0 to 76 should hit a tab
        for col in 0..=76 {
            assert!(
                tab_bar_hit_test(col).is_some(),
                "Column {col} should hit a tab, but returned None"
            );
        }
    }

    #[test]
    fn tab_bar_hit_test_sequential() {
        // Verify tabs are selected in correct order as we scan left to right
        let mut last_tab_index = None;
        for col in 0..=76 {
            if let Some(tab) = tab_bar_hit_test(col) {
                let tab_index = tab.index();
                if let Some(last_index) = last_tab_index {
                    // Tab index should either stay the same or increment by 1
                    assert!(
                        tab_index == last_index || tab_index == last_index + 1,
                        "Tab order violation at col {col}: jumped from tab {last_index} to {tab_index}"
                    );
                }
                last_tab_index = Some(tab_index);
            }
        }
    }

    #[test]
    fn tab_bar_hit_test_realistic_click_positions() {
        // Test realistic positions where users might click on each tab
        // (roughly the visual center of each label)
        assert_eq!(tab_bar_hit_test(6), Some(TabId::Explorer)); // "plorer"
        assert_eq!(tab_bar_hit_test(18), Some(TabId::Graph)); // "raph"
        assert_eq!(tab_bar_hit_test(31), Some(TabId::Timeline)); // "meline"
        assert_eq!(tab_bar_hit_test(43), Some(TabId::Play)); // "lay"
        assert_eq!(tab_bar_hit_test(53), Some(TabId::Solo)); // "lo"
        assert_eq!(tab_bar_hit_test(64), Some(TabId::Sheet)); // "heet"
        assert_eq!(tab_bar_hit_test(74), Some(TabId::Dice)); // "ce"
    }
}
