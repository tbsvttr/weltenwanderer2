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

/// Trait that all screens implement.
pub trait Screen {
    /// Update state based on input. Returns a transition if the screen should change.
    fn update(&mut self, app: &mut AppState) -> Transition;
    /// Draw the screen.
    fn draw(&self, app: &AppState);
}
