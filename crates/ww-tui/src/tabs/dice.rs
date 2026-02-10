//! Dice roller tab.

use rand::SeedableRng;
use rand::rngs::StdRng;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use ww_mechanics::{DicePool, Die, RollResult};

use super::{InputMode, Tab};

/// Available die types in order.
const DIE_TYPES: &[(u32, &str)] = &[
    (4, "d4"),
    (6, "d6"),
    (8, "d8"),
    (10, "d10"),
    (12, "d12"),
    (20, "d20"),
    (100, "d100"),
];

/// Dice roller tab state.
pub struct DiceTab {
    /// Number of dice in the pool.
    pool_size: u32,
    /// Index into DIE_TYPES.
    die_index: usize,
    /// Last roll result.
    result: Option<RollResult>,
    /// RNG used for rolling.
    rng: StdRng,
}

impl DiceTab {
    /// Create a new dice tab with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            pool_size: 2,
            die_index: 5, // d20
            result: None,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn die_sides(&self) -> u32 {
        DIE_TYPES[self.die_index].0
    }

    fn die_label(&self) -> &str {
        DIE_TYPES[self.die_index].1
    }

    fn roll(&mut self) {
        let die = die_from_sides(self.die_sides());
        let pool = DicePool::new().add(die, self.pool_size);
        self.result = Some(pool.roll(&mut self.rng));
    }
}

/// Convert a sides count to a `Die` variant.
fn die_from_sides(sides: u32) -> Die {
    match sides {
        4 => Die::D4,
        6 => Die::D6,
        8 => Die::D8,
        10 => Die::D10,
        12 => Die::D12,
        20 => Die::D20,
        100 => Die::D100,
        n => Die::Custom(n),
    }
}

impl Tab for DiceTab {
    fn input_mode(&self) -> InputMode {
        InputMode::VimNav
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.die_index > 0 {
                    self.die_index -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.die_index + 1 < DIE_TYPES.len() {
                    self.die_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.pool_size = (self.pool_size + 1).min(20);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.pool_size = self.pool_size.saturating_sub(1).max(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.roll();
            }
            _ => {}
        }
        false
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Dice Roller ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 4 || inner.width < 20 {
            return;
        }

        let mut lines: Vec<Line<'static>> = Vec::new();

        // Pool description
        lines.push(Line::from(vec![
            Span::styled("Pool: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}{}", self.pool_size, self.die_label()),
                Style::default().fg(Color::Yellow).bold(),
            ),
        ]));
        lines.push(Line::from(""));

        // Die type selector
        let die_spans: Vec<Span<'static>> = DIE_TYPES
            .iter()
            .enumerate()
            .flat_map(|(i, (_, label))| {
                let style = if i == self.die_index {
                    Style::default().fg(Color::Black).bg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                };
                vec![Span::styled(format!(" {label} "), style), Span::raw(" ")]
            })
            .collect();
        lines.push(Line::from(vec![Span::styled(
            "Die:   ",
            Style::default().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(die_spans));
        lines.push(Line::from(""));

        // Pool size
        lines.push(Line::from(vec![
            Span::styled("Count: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.pool_size),
                Style::default().fg(Color::White).bold(),
            ),
            Span::styled(
                "  (\u{2191}/\u{2193} to adjust)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        // Roll prompt
        lines.push(Line::from(Span::styled(
            "Press Enter or Space to roll!",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(""));

        // Result display
        if let Some(result) = &self.result {
            lines.push(Line::from(Span::styled(
                "Result:",
                Style::default().fg(Color::DarkGray),
            )));

            // Individual dice
            let dice_spans: Vec<Span<'static>> = result
                .dice
                .iter()
                .flat_map(|d| {
                    vec![Span::styled(
                        format!(" [{}] ", d.value),
                        Style::default().fg(Color::Yellow).bold(),
                    )]
                })
                .collect();
            lines.push(Line::from(dice_spans));
            lines.push(Line::from(""));

            // Total
            lines.push(Line::from(vec![
                Span::styled("Total: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", result.total()),
                    Style::default().fg(Color::Green).bold(),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn status_hint(&self) -> &str {
        "\u{2190}/\u{2192}:die type  \u{2191}/\u{2193}:count  Enter/Space:roll  Tab:view  ?:help  q:quit"
    }
}
