pub mod entity_detail;
pub mod entity_list;
pub mod graph_view;
pub mod timeline_view;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};

use super::app::{ActiveView, App, InputMode};

pub fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Entities", "[2] Graph", "[3] Timeline"];
    let selected = match app.active_view {
        ActiveView::EntityList | ActiveView::EntityDetail => 0,
        ActiveView::Graph => 1,
        ActiveView::Timeline => 2,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::White).bold())
        .divider(" | ");

    frame.render_widget(tabs, area);
}

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = match app.input_mode {
        InputMode::Search => {
            format!("/{} (Enter to confirm, Esc to cancel)", app.search_query)
        }
        InputMode::Normal => match app.active_view {
            ActiveView::EntityList => {
                format!(
                    "'{}' | {} entities | j/k:navigate Enter:select /:search Tab:view ?:help q:quit",
                    app.world.meta.name,
                    app.filtered_ids.len()
                )
            }
            ActiveView::EntityDetail => {
                let name = app
                    .detail_entity_id
                    .map(|id| app.world.entity_name(id))
                    .unwrap_or("<unknown>");
                format!("{name} | j/k:scroll Esc:back ?:help q:quit")
            }
            ActiveView::Graph => {
                format!(
                    "Graph | {} relationships | j/k:scroll Tab:view ?:help q:quit",
                    app.world.relationship_count()
                )
            }
            ActiveView::Timeline => {
                "Timeline | j/k:navigate Enter:select Tab:view ?:help q:quit".to_string()
            }
        },
    };

    let bar = Paragraph::new(status).style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(bar, area);
}

pub fn draw_help_popup(frame: &mut Frame) {
    let area = centered_rect(50, 60, frame.area());

    let help_text = vec![
        Line::from("Keyboard Shortcuts").style(Style::default().bold()),
        Line::from(""),
        Line::from("  j / ↓       Move down"),
        Line::from("  k / ↑       Move up"),
        Line::from("  g           Go to top"),
        Line::from("  G           Go to bottom"),
        Line::from("  Enter       Select / drill in"),
        Line::from("  Esc         Go back"),
        Line::from("  /           Search (filter by name)"),
        Line::from("  Tab         Next view"),
        Line::from("  1 / 2 / 3   Switch view"),
        Line::from("  ?           Toggle this help"),
        Line::from("  q           Quit"),
        Line::from("  Ctrl+C      Force quit"),
    ];

    let popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
