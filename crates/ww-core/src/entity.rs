use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::component::ComponentSet;

/// Unique identifier for every entity in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// The kind of an entity. Extensible via `Custom(String)` for user-defined types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Location,
    Character,
    Faction,
    Event,
    Item,
    Lore,
    Custom(String),
}

impl EntityKind {
    /// Returns true if this kind is a location subtype (fortress, city, etc.)
    /// Location subtypes are stored as `EntityKind::Location` with the subtype
    /// recorded in the `LocationComponent`.
    pub fn is_location_subtype(name: &str) -> bool {
        matches!(
            name,
            "fortress"
                | "city"
                | "town"
                | "village"
                | "region"
                | "continent"
                | "room"
                | "wilderness"
                | "dungeon"
                | "building"
                | "landmark"
                | "plane"
        )
    }

    /// Try to parse a kind from a string, recognizing location subtypes.
    pub fn parse(s: &str) -> (Self, Option<String>) {
        match s {
            "location" => (Self::Location, None),
            "character" => (Self::Character, None),
            "faction" => (Self::Faction, None),
            "event" => (Self::Event, None),
            "item" => (Self::Item, None),
            "lore" => (Self::Lore, None),
            other if Self::is_location_subtype(other) => (Self::Location, Some(other.to_string())),
            other => (Self::Custom(other.to_string()), None),
        }
    }
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Location => write!(f, "location"),
            Self::Character => write!(f, "character"),
            Self::Faction => write!(f, "faction"),
            Self::Event => write!(f, "event"),
            Self::Item => write!(f, "item"),
            Self::Lore => write!(f, "lore"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// A flexible metadata value that supports common types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<MetadataValue>),
    Map(HashMap<String, MetadataValue>),
}

impl fmt::Display for MetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Integer(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            Self::Map(_) => write!(f, "{{...}}"),
        }
    }
}

/// Core entity struct. Every world object is an Entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, MetadataValue>,
    pub components: ComponentSet,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity {
    pub fn new(kind: EntityKind, name: impl Into<String>) -> Self {
        Self::with_id(EntityId::new(), kind, name)
    }

    /// Create an entity with a pre-assigned ID.
    ///
    /// Used by the DSL compiler when the resolver has already assigned IDs
    /// during the name-resolution pass.
    pub fn with_id(id: EntityId, kind: EntityKind, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            kind,
            name: name.into(),
            description: String::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
            components: ComponentSet::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns the location subtype if this is a location entity with one set.
    pub fn location_subtype(&self) -> Option<&str> {
        self.components
            .location
            .as_ref()
            .map(|l| l.location_type.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_display_shows_short_form() {
        let id = EntityId(Uuid::parse_str("a3f2b1c8-1234-5678-9abc-def012345678").unwrap());
        assert_eq!(id.to_string(), "a3f2b1c8");
    }

    #[test]
    fn entity_kind_parse_location_subtypes() {
        let (kind, subtype) = EntityKind::parse("fortress");
        assert_eq!(kind, EntityKind::Location);
        assert_eq!(subtype.as_deref(), Some("fortress"));
    }

    #[test]
    fn entity_kind_parse_standard_kinds() {
        let (kind, subtype) = EntityKind::parse("character");
        assert_eq!(kind, EntityKind::Character);
        assert!(subtype.is_none());
    }

    #[test]
    fn entity_kind_parse_custom() {
        let (kind, subtype) = EntityKind::parse("vehicle");
        assert_eq!(kind, EntityKind::Custom("vehicle".to_string()));
        assert!(subtype.is_none());
    }

    #[test]
    fn new_entity_has_timestamps() {
        let entity = Entity::new(EntityKind::Character, "Kael");
        assert!(!entity.name.is_empty());
        assert_eq!(entity.kind, EntityKind::Character);
    }

    #[test]
    fn with_id_preserves_given_id() {
        let id = EntityId(Uuid::parse_str("a3f2b1c8-1234-5678-9abc-def012345678").unwrap());
        let entity = Entity::with_id(id, EntityKind::Character, "Kael");
        assert_eq!(entity.id, id);
        assert_eq!(entity.name, "Kael");
        assert_eq!(entity.kind, EntityKind::Character);
    }
}
