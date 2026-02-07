//! Error types for the solo TTRPG engine.

use thiserror::Error;

/// Result type for solo operations.
pub type SoloResult<T> = Result<T, SoloError>;

/// Errors that can occur during a solo TTRPG session.
#[derive(Debug, Error)]
pub enum SoloError {
    /// No active scene to operate on.
    #[error("no active scene")]
    NoActiveScene,

    /// Invalid likelihood string.
    #[error("invalid likelihood: {0}")]
    InvalidLikelihood(String),

    /// Invalid choice or input.
    #[error("invalid choice: {0}")]
    InvalidChoice(String),

    /// No threads to reference.
    #[error("no threads to reference")]
    NoThreads,

    /// No NPCs to reference.
    #[error("no NPCs to reference")]
    NoNpcs,

    /// Unknown command.
    #[error("unknown command: {0}")]
    UnknownCommand(String),

    /// Fiction engine error.
    #[error("{0}")]
    Fiction(#[from] ww_fiction::FictionError),
}
