use ww_core::entity::EntityId;

/// A type alias for `Result<T, SimError>`.
pub type SimResult<T> = Result<T, SimError>;

/// Errors that can occur during simulation.
#[derive(Debug, thiserror::Error)]
pub enum SimError {
    /// The requested entity was not found in the simulation.
    #[error("entity not found in simulation: {0}")]
    EntityNotFound(EntityId),

    /// No traversable path exists between two locations.
    #[error("no path found from {from} to {to}")]
    NoPath {
        /// The starting location.
        from: EntityId,
        /// The target location.
        to: EntityId,
    },

    /// A generic error from a simulation system.
    #[error("system error: {0}")]
    SystemError(String),
}
