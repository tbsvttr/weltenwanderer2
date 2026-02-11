//! Error types for the fiction engine.

use thiserror::Error;
use ww_core::EntityId;

/// Result type for fiction operations.
pub type FictionResult<T> = Result<T, FictionError>;

/// Errors that can occur during interactive fiction.
#[derive(Debug, Error)]
pub enum FictionError {
    /// Player entity not found in world.
    #[error("player entity not found: {0}")]
    PlayerNotFound(EntityId),

    /// Target entity not found.
    #[error("entity not found: {0}")]
    EntityNotFound(String),

    /// Entity exists in the world but is not at the player's location.
    #[error("{0} is not here.")]
    EntityNotHere(String),

    /// Location not found or invalid.
    #[error("location not found: {0}")]
    LocationNotFound(String),

    /// No path exists between locations.
    #[error("no path from {from} to {to}")]
    NoPath {
        /// Starting location.
        from: String,
        /// Target location.
        to: String,
    },

    /// Invalid command input.
    #[error("unknown command: {0}")]
    UnknownCommand(String),

    /// Dialogue not found.
    #[error("dialogue not found: {0}")]
    DialogueNotFound(String),

    /// Invalid choice selection.
    #[error("invalid choice: {0}")]
    InvalidChoice(usize),

    /// Condition not met for action.
    #[error("condition not met: {0}")]
    ConditionNotMet(String),

    /// Item not in inventory.
    #[error("item not in inventory: {0}")]
    ItemNotInInventory(String),

    /// Cannot take item.
    #[error("cannot take: {0}")]
    CannotTake(String),

    /// Simulation error.
    #[error("simulation error: {0}")]
    Simulation(#[from] ww_simulation::SimError),
}
