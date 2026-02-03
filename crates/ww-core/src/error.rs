use crate::entity::{EntityId, EntityKind};
use crate::relationship::RelationshipId;

pub type WwResult<T> = Result<T, WwError>;

#[derive(Debug, thiserror::Error)]
pub enum WwError {
    #[error("entity not found: {0}")]
    EntityNotFound(EntityId),

    #[error("entity already exists: \"{0}\"")]
    DuplicateName(String),

    #[error("relationship not found: {0}")]
    RelationshipNotFound(RelationshipId),

    #[error("invalid reference: entity \"{name}\" of kind {expected_kind:?} not found")]
    InvalidReference {
        name: String,
        expected_kind: Option<EntityKind>,
    },

    #[error("validation error: {0}")]
    Validation(String),
}
