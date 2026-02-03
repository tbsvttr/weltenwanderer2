//! Core types for Weltenwanderer: entities, relationships, and the world model.
//!
//! This crate defines the data model that the DSL compiles into. It is
//! independent of the parser — you can construct a [`World`] programmatically
//! or deserialize one from JSON.

pub mod component;
pub mod entity;
pub mod error;
pub mod query;
pub mod relationship;
pub mod timeline;
pub mod world;

pub use entity::{Entity, EntityId, EntityKind};
pub use error::{WwError, WwResult};
pub use relationship::{Relationship, RelationshipId, RelationshipKind};
pub use world::{World, WorldMeta};
