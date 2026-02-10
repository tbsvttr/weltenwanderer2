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
    // The ratatui Tabs widget renders these sequentially with the divider between them.
    // We need to account for the exact rendering including dividers.

    let labels = [
        ("[1]Explorer", 11), // 11 chars
        ("[2]Graph", 9),     // 9 chars
        ("[3]Timeline", 12), // 12 chars
        ("[4]Play", 8),      // 8 chars
        ("[5]Solo", 8),      // 8 chars
        ("[6]Sheet", 9),     // 9 chars
        ("[7]Dice", 8),      // 8 chars
    ];

    let divider_len = 3u16; // " | " is 3 chars

    let mut x = 0u16;
    for (i, (_, len)) in labels.iter().enumerate() {
        let end_x = x + len;

        // Check if click is within this tab's label area
        if col >= x && col < end_x {
            return Some(TabId::ALL[i]);
        }

        // Move past the label and divider (except for last tab)
        x = end_x;
        if i < labels.len() - 1 {
            x += divider_len;
        }
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
