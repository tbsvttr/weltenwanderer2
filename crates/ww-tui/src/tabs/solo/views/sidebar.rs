//! Sidebar panel: chaos, scene, tracks, threads, NPCs.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};

use crate::tabs::solo::SoloTab;

/// Draw the sidebar status panel.
pub fn draw(frame: &mut Frame, tab: &SoloTab, area: Rect) {
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 || inner.width < 6 {
        return;
    }

    let track_count = tab
        .session
        .sheet()
        .map(|s| s.tracks.len() as u16)
        .unwrap_or(0);

    // Only show chaos/scene section if chaos is enabled
    if tab.session.world_config().enable_chaos {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),                  // Chaos + Scene
                Constraint::Length(track_count.max(1)), // Tracks
                Constraint::Min(2),                     // Threads + NPCs
            ])
            .split(inner);

        draw_info(frame, tab, chunks[0]);
        draw_tracks(frame, tab, chunks[1]);
        draw_lists(frame, tab, chunks[2]);
    } else {
        // Chaos disabled: skip chaos/scene section
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(track_count.max(1)), // Tracks
                Constraint::Min(2),                     // Threads + NPCs
            ])
            .split(inner);

        draw_tracks(frame, tab, chunks[0]);
        draw_lists(frame, tab, chunks[1]);
    }
}

/// Chaos factor and scene info.
fn draw_info(frame: &mut Frame, tab: &SoloTab, area: Rect) {
    let chaos = tab.session.chaos().value();
    let chaos_label = tab
        .session
        .world_config()
        .chaos_label
        .as_deref()
        .unwrap_or("Chaos");

    let chaos_color = if chaos >= 7 {
        Color::Red
    } else if chaos >= 4 {
        Color::Yellow
    } else {
        Color::Green
    };

    let mut lines = vec![Line::from(vec![
        Span::raw(format!("{chaos_label}: ")),
        Span::styled(
            format!("{chaos}/9"),
            Style::default().fg(chaos_color).bold(),
        ),
    ])];

    if let Some(scene) = tab.session.current_scene() {
        lines.push(Line::from(vec![
            Span::raw(format!("Scene #{} ", scene.number)),
            Span::styled(
                format!("{}", scene.status),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "No active scene",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let p = Paragraph::new(lines);
    frame.render_widget(p, area);
}

/// Track gauges (HP, Stress, Wounds, etc.).
fn draw_tracks(frame: &mut Frame, tab: &SoloTab, area: Rect) {
    let Some(sheet) = tab.session.sheet() else {
        let p = Paragraph::new(Span::styled(
            "No character",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(p, area);
        return;
    };

    let mut tracks: Vec<_> = sheet.tracks.values().collect();
    tracks.sort_by(|a, b| a.name.cmp(&b.name));

    for (i, track) in tracks.iter().enumerate() {
        if i as u16 >= area.height {
            break;
        }

        let row = Rect::new(area.x, area.y + i as u16, area.width, 1);
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

/// Thread and NPC lists.
fn draw_lists(frame: &mut Frame, tab: &SoloTab, area: Rect) {
    let threads = tab.session.threads().active();
    let npcs = tab.session.npcs().list();

    let thread_height = (threads.len() as u16 + 1).min(area.height / 2).max(1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(thread_height), Constraint::Min(1)])
        .split(area);

    // Threads
    let thread_items: Vec<ListItem> = threads
        .iter()
        .map(|t| {
            ListItem::new(Line::from(Span::styled(
                format!(" \u{25b8} {}", t.name),
                Style::default().fg(Color::Cyan),
            )))
        })
        .collect();

    let thread_list = List::new(thread_items).block(
        Block::default()
            .title("Threads")
            .borders(Borders::NONE)
            .title_style(Style::default().fg(Color::Cyan).bold()),
    );
    frame.render_widget(thread_list, chunks[0]);

    // NPCs
    let npc_items: Vec<ListItem> = npcs
        .iter()
        .map(|n| {
            ListItem::new(Line::from(Span::styled(
                format!(" \u{25b8} {}", n.name),
                Style::default().fg(Color::Magenta),
            )))
        })
        .collect();

    let npc_list = List::new(npc_items).block(
        Block::default()
            .title("NPCs")
            .borders(Borders::NONE)
            .title_style(Style::default().fg(Color::Magenta).bold()),
    );
    frame.render_widget(npc_list, chunks[1]);
}
