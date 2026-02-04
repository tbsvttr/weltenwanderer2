use crate::entity::{EntityId, EntityKind};
use crate::relationship::RelationshipId;

/// Alias for `Result<T, WwError>`.
pub type WwResult<T> = Result<T, WwError>;

/// Errors that can occur when manipulating a world.
#[derive(Debug, thiserror::Error)]
pub enum WwError {
    /// The requested entity ID does not exist in the world.
    #[error("entity not found: {0}")]
    EntityNotFound(EntityId),

    /// An entity with the same name already exists.
    #[error("entity already exists: \"{0}\"")]
    DuplicateName(String),

    /// The requested relationship ID does not exist in the world.
    #[error("relationship not found: {0}")]
    RelationshipNotFound(RelationshipId),

    /// A named reference could not be resolved to an existing entity.
    #[error("invalid reference: entity \"{name}\" of kind {expected_kind:?} not found")]
    InvalidReference {
        /// The unresolved entity name.
        name: String,
        /// The expected entity kind, if known.
        expected_kind: Option<EntityKind>,
    },

    /// A generic validation error with a descriptive message.
    #[error("validation error: {0}")]
    Validation(String),
}
