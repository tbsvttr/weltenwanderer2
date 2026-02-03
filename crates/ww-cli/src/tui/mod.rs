mod app;
mod views;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::prelude::*;

use ww_core::World;

use app::{App, ActiveView, InputMode};

pub fn run(world: World) -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("terminal error: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| format!("terminal error: {e}"))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("terminal error: {e}"))?;

    let mut app = App::new(world);

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| draw(frame, app))
            .map_err(|e| format!("draw error: {e}"))?;

        if let Event::Key(key) = event::read().map_err(|e| format!("event error: {e}"))? {
            // Ctrl+C always quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(());
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                    KeyCode::Char('g') => app.move_to_top(),
                    KeyCode::Char('G') => app.move_to_bottom(),
                    KeyCode::Enter => app.select(),
                    KeyCode::Esc => app.go_back(),
                    KeyCode::Char('/') => app.start_search(),
                    KeyCode::Tab => app.next_view(),
                    KeyCode::Char('1') => app.switch_view(ActiveView::EntityList),
                    KeyCode::Char('2') => app.switch_view(ActiveView::Graph),
                    KeyCode::Char('3') => app.switch_view(ActiveView::Timeline),
                    KeyCode::Char('?') => app.toggle_help(),
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Esc => app.cancel_search(),
                    KeyCode::Enter => app.confirm_search(),
                    KeyCode::Backspace => app.search_backspace(),
                    KeyCode::Char(c) => app.search_push(c),
                    _ => {}
                },
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),   // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    views::draw_tab_bar(frame, app, chunks[0]);

    match app.active_view {
        ActiveView::EntityList => views::entity_list::draw(frame, app, chunks[1]),
        ActiveView::EntityDetail => views::entity_detail::draw(frame, app, chunks[1]),
        ActiveView::Graph => views::graph_view::draw(frame, app, chunks[1]),
        ActiveView::Timeline => views::timeline_view::draw(frame, app, chunks[1]),
    }

    views::draw_status_bar(frame, app, chunks[2]);

    if app.show_help {
        views::draw_help_popup(frame);
    }
}
