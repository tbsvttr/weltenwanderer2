use ww_core::entity::EntityId;

pub type SimResult<T> = Result<T, SimError>;

#[derive(Debug, thiserror::Error)]
pub enum SimError {
    #[error("entity not found in simulation: {0}")]
    EntityNotFound(EntityId),

    #[error("no path found from {from} to {to}")]
    NoPath { from: EntityId, to: EntityId },

    #[error("system error: {0}")]
    SystemError(String),
}
