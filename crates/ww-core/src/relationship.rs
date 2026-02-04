use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{EntityId, MetadataValue};

/// Unique identifier for a relationship edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipId(pub Uuid);

impl RelationshipId {
    /// Generates a new random relationship identifier.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RelationshipId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RelationshipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// A directed edge between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier for this relationship edge.
    pub id: RelationshipId,
    /// The entity where this relationship originates.
    pub source: EntityId,
    /// The entity this relationship points to.
    pub target: EntityId,
    /// The semantic kind of this relationship.
    pub kind: RelationshipKind,
    /// Optional human-readable label describing the relationship.
    pub label: Option<String>,
    /// Arbitrary key-value metadata attached to this relationship.
    pub metadata: HashMap<String, MetadataValue>,
    /// Whether the relationship applies in both directions.
    pub bidirectional: bool,
}

impl Relationship {
    /// Creates a new relationship from `source` to `target` with the given kind.
    pub fn new(source: EntityId, kind: RelationshipKind, target: EntityId) -> Self {
        let bidirectional = kind.is_bidirectional();
        Self {
            id: RelationshipId::new(),
            source,
            target,
            kind,
            label: None,
            metadata: HashMap::new(),
            bidirectional,
        }
    }

    /// Attaches a human-readable label to this relationship.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// The kind of relationship between two entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    // Spatial
    /// Entity is physically inside another entity.
    ContainedIn,
    /// Entity has a path or passage to another entity.
    ConnectedTo,
    /// Entity is situated at a location.
    LocatedAt,
    /// Entity uses a location as its home base.
    BasedAt,

    // Social
    /// Entity belongs to an organization or group.
    MemberOf,
    /// Entity leads or commands another entity.
    LeaderOf,
    /// Entities are allied or cooperating.
    AlliedWith,
    /// Entities are rivals or adversaries.
    RivalOf,
    /// Entities share a familial or generic association.
    RelatedTo,

    // Ownership
    /// Entity is owned or possessed by another entity.
    OwnedBy,

    // Events
    /// Entity took part in an event.
    ParticipatedIn,
    /// Entity was caused or triggered by another entity.
    CausedBy,

    // Lore
    /// Entity references or cites another entity.
    References,

    // Generic
    /// A user-defined relationship kind.
    Custom(String),
}

impl RelationshipKind {
    /// Returns true if this relationship kind is inherently bidirectional.
    pub fn is_bidirectional(&self) -> bool {
        matches!(
            self,
            Self::AlliedWith | Self::RivalOf | Self::RelatedTo | Self::ConnectedTo
        )
    }

    /// Returns the human-readable DSL phrase for this relationship kind.
    pub fn as_phrase(&self) -> &str {
        match self {
            Self::ContainedIn => "in",
            Self::ConnectedTo => "connected to",
            Self::LocatedAt => "located at",
            Self::BasedAt => "based at",
            Self::MemberOf => "member of",
            Self::LeaderOf => "led by",
            Self::AlliedWith => "allied with",
            Self::RivalOf => "rival of",
            Self::RelatedTo => "related to",
            Self::OwnedBy => "owned by",
            Self::ParticipatedIn => "involving",
            Self::CausedBy => "caused by",
            Self::References => "references",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for RelationshipKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_phrase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bidirectional_kinds() {
        assert!(RelationshipKind::AlliedWith.is_bidirectional());
        assert!(RelationshipKind::ConnectedTo.is_bidirectional());
        assert!(!RelationshipKind::MemberOf.is_bidirectional());
        assert!(!RelationshipKind::OwnedBy.is_bidirectional());
    }

    #[test]
    fn relationship_builder() {
        let src = EntityId::new();
        let tgt = EntityId::new();
        let rel =
            Relationship::new(src, RelationshipKind::AlliedWith, tgt).with_label("trusted ally");
        assert!(rel.bidirectional);
        assert_eq!(rel.label.as_deref(), Some("trusted ally"));
    }
}
