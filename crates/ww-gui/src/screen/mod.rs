//! Screen management: trait definition, screen identifiers, and transitions.

pub mod dice;
pub mod explorer;
pub mod graph;
pub mod play;
pub mod sheet;
pub mod solo;
pub mod timeline;
pub mod title;

use crate::app::AppState;

/// Identifies which screen to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    /// Title / world selector screen.
    Title,
    /// World explorer (entity list + detail).
    Explorer,
    /// Relationship graph view.
    Graph,
    /// Timeline view.
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

/// A transition between screens.
#[derive(Debug, Clone)]
pub enum Transition {
    /// Push a new screen onto the stack.
    Push(ScreenId),
    /// Pop the current screen and return to the previous one.
    Pop,
    /// Replace the current screen.
    Replace(ScreenId),
    /// No transition.
    None,
}

/// Tab labels shared across all tabbed screens.
pub const TAB_LABELS: &[&str] = &["Entities", "Graph", "Timeline", "Play", "Solo"];

/// Number of tabs in the tab bar.
pub const TAB_COUNT: f32 = 5.0;

/// Handle a tab click and return a transition if a different tab was clicked.
///
/// `active` is the index of the currently active tab.
/// Returns `Some(transition)` if the user clicked a different tab.
pub fn handle_tab_click(mx: f32, my: f32, active: usize) -> Option<Transition> {
    use macroquad::prelude::*;
    if !is_mouse_button_pressed(MouseButton::Left) || my >= 14.0 {
        return None;
    }
    let tab_idx = (mx / (crate::theme::CANVAS_W / TAB_COUNT)) as usize;
    if tab_idx == active {
        return None;
    }
    match tab_idx {
        0 => Some(Transition::Replace(ScreenId::Explorer)),
        1 => Some(Transition::Replace(ScreenId::Graph)),
        2 => Some(Transition::Replace(ScreenId::Timeline)),
        3 => Some(Transition::Push(ScreenId::Play)),
        4 => Some(Transition::Push(ScreenId::Solo)),
        _ => None,
    }
}

/// Trait that all screens implement.
pub trait Screen {
    /// Update state based on input. Returns a transition if the screen should change.
    fn update(&mut self, app: &mut AppState) -> Transition;
    /// Draw the screen.
    fn draw(&self, app: &AppState);
}
