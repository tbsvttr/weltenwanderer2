//! Error types for the mechanics engine.

/// Errors that can occur during mechanics operations.
#[derive(Debug, thiserror::Error)]
pub enum MechError {
    /// An attribute referenced in a check does not exist in the ruleset.
    #[error("unknown attribute: {0}")]
    UnknownAttribute(String),

    /// A skill referenced in a check does not exist in the ruleset.
    #[error("unknown skill: {0}")]
    UnknownSkill(String),

    /// A track was not found on the character sheet.
    #[error("track '{0}' not found")]
    TrackNotFound(String),

    /// A dice pool configuration is invalid.
    #[error("invalid pool: {0}")]
    InvalidPool(String),

    /// No mechanics configuration was found in the world.
    #[error("no mechanics config found in world")]
    NoMechanicsConfig,

    /// The mechanics configuration in the world is malformed.
    #[error("invalid mechanics config: {0}")]
    InvalidConfig(String),

    /// An error occurred during combat resolution.
    #[error("combat error: {0}")]
    CombatError(String),

    /// No participant is currently active in combat.
    #[error("no active participant")]
    NoActiveParticipant,
}

/// Convenience result type for mechanics operations.
pub type MechResult<T> = Result<T, MechError>;
