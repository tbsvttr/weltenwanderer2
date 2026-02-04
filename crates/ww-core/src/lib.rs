//! Core types for Weltenwanderer: entities, relationships, and the world model.
//!
//! This crate defines the data model that the DSL compiles into. It is
//! independent of the parser — you can construct a [`World`] programmatically
//! or deserialize one from JSON.

/// Typed component data (location, character stats, event details, etc.).
pub mod component;
/// Entity types, identifiers, and metadata values.
pub mod entity;
/// Error types used throughout the crate.
pub mod error;
/// Query builder for filtering and searching entities.
pub mod query;
/// Relationship types and identifiers connecting entities.
pub mod relationship;
/// Chronological timeline built from event entities.
pub mod timeline;
/// The central world model that owns entities and relationships.
pub mod world;

/// Re-export core entity types.
pub use entity::{Entity, EntityId, EntityKind};
/// Re-export error types.
pub use error::{WwError, WwResult};
/// Re-export relationship types.
pub use relationship::{Relationship, RelationshipId, RelationshipKind};
/// Re-export world model types.
pub use world::{World, WorldMeta};
