use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::{Entity, EntityId, EntityKind, MetadataValue};
use crate::error::{WwError, WwResult};
use crate::query::QueryBuilder;
use crate::relationship::{Relationship, RelationshipId};

/// Metadata about the world itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMeta {
    pub name: String,
    pub description: String,
    pub genre: Option<String>,
    pub setting: Option<String>,
    pub authors: Vec<String>,
    pub schema_version: u32,
    pub properties: HashMap<String, MetadataValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorldMeta {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            description: String::new(),
            genre: None,
            setting: None,
            authors: Vec::new(),
            schema_version: 1,
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// The central world model. Owns all entities and relationships.
#[derive(Debug, Clone)]
pub struct World {
    pub meta: WorldMeta,
    entities: HashMap<EntityId, Entity>,
    relationships: HashMap<RelationshipId, Relationship>,

    // Indexes
    by_kind: HashMap<EntityKind, Vec<EntityId>>,
    by_name_lower: HashMap<String, EntityId>,
    edges_from: HashMap<EntityId, Vec<RelationshipId>>,
    edges_to: HashMap<EntityId, Vec<RelationshipId>>,
}

impl World {
    pub fn new(meta: WorldMeta) -> Self {
        Self {
            meta,
            entities: HashMap::new(),
            relationships: HashMap::new(),
            by_kind: HashMap::new(),
            by_name_lower: HashMap::new(),
            edges_from: HashMap::new(),
            edges_to: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Entity CRUD
    // -----------------------------------------------------------------------

    /// Add an entity to the world. Returns the entity's ID.
    pub fn add_entity(&mut self, entity: Entity) -> WwResult<EntityId> {
        let name_lower = entity.name.to_lowercase();
        if self.by_name_lower.contains_key(&name_lower) {
            return Err(WwError::DuplicateName(entity.name.clone()));
        }

        let id = entity.id;
        self.by_kind
            .entry(entity.kind.clone())
            .or_default()
            .push(id);
        self.by_name_lower.insert(name_lower, id);
        self.entities.insert(id, entity);
        Ok(id)
    }

    /// Get a reference to an entity by ID.
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Get a mutable reference to an entity by ID.
    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// Find an entity by name (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Option<&Entity> {
        self.by_name_lower
            .get(&name.to_lowercase())
            .and_then(|id| self.entities.get(id))
    }

    /// Find an entity ID by name (case-insensitive).
    pub fn find_id_by_name(&self, name: &str) -> Option<EntityId> {
        self.by_name_lower.get(&name.to_lowercase()).copied()
    }

    /// Remove an entity and all its relationships.
    pub fn remove_entity(&mut self, id: EntityId) -> WwResult<Entity> {
        let entity = self
            .entities
            .remove(&id)
            .ok_or(WwError::EntityNotFound(id))?;

        // Remove from indexes
        let name_lower = entity.name.to_lowercase();
        self.by_name_lower.remove(&name_lower);
        if let Some(ids) = self.by_kind.get_mut(&entity.kind) {
            ids.retain(|eid| *eid != id);
        }

        // Remove all relationships involving this entity
        let rel_ids: Vec<RelationshipId> = self
            .relationships
            .values()
            .filter(|r| r.source == id || r.target == id)
            .map(|r| r.id)
            .collect();
        for rid in rel_ids {
            self.remove_relationship_internal(rid);
        }

        Ok(entity)
    }

    // -----------------------------------------------------------------------
    // Relationship CRUD
    // -----------------------------------------------------------------------

    /// Add a relationship between two entities.
    pub fn add_relationship(&mut self, rel: Relationship) -> WwResult<RelationshipId> {
        if !self.entities.contains_key(&rel.source) {
            return Err(WwError::EntityNotFound(rel.source));
        }
        if !self.entities.contains_key(&rel.target) {
            return Err(WwError::EntityNotFound(rel.target));
        }

        let id = rel.id;
        self.edges_from.entry(rel.source).or_default().push(id);
        self.edges_to.entry(rel.target).or_default().push(id);

        // For bidirectional relationships, also index the reverse direction
        if rel.bidirectional {
            self.edges_from.entry(rel.target).or_default().push(id);
            self.edges_to.entry(rel.source).or_default().push(id);
        }

        self.relationships.insert(id, rel);
        Ok(id)
    }

    /// Remove a relationship by ID.
    pub fn remove_relationship(&mut self, id: RelationshipId) -> WwResult<Relationship> {
        let rel = self
            .relationships
            .get(&id)
            .ok_or(WwError::RelationshipNotFound(id))?
            .clone();
        self.remove_relationship_internal(id);
        Ok(rel)
    }

    fn remove_relationship_internal(&mut self, id: RelationshipId) {
        if let Some(rel) = self.relationships.remove(&id) {
            if let Some(ids) = self.edges_from.get_mut(&rel.source) {
                ids.retain(|rid| *rid != id);
            }
            if let Some(ids) = self.edges_to.get_mut(&rel.target) {
                ids.retain(|rid| *rid != id);
            }
            if rel.bidirectional {
                if let Some(ids) = self.edges_from.get_mut(&rel.target) {
                    ids.retain(|rid| *rid != id);
                }
                if let Some(ids) = self.edges_to.get_mut(&rel.source) {
                    ids.retain(|rid| *rid != id);
                }
            }
        }
    }

    /// Get all relationships originating from an entity.
    pub fn relationships_from(&self, entity: EntityId) -> Vec<&Relationship> {
        self.edges_from
            .get(&entity)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.relationships.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all relationships pointing to an entity.
    pub fn relationships_to(&self, entity: EntityId) -> Vec<&Relationship> {
        self.edges_to
            .get(&entity)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.relationships.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all relationships involving an entity (either direction).
    pub fn relationships_of(&self, entity: EntityId) -> Vec<&Relationship> {
        let mut rels: Vec<&Relationship> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for rel in self.relationships_from(entity) {
            if seen.insert(rel.id) {
                rels.push(rel);
            }
        }
        for rel in self.relationships_to(entity) {
            if seen.insert(rel.id) {
                rels.push(rel);
            }
        }
        rels
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Get all entities of a specific kind.
    pub fn entities_by_kind(&self, kind: &EntityKind) -> Vec<&Entity> {
        self.by_kind
            .get(kind)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all entities.
    pub fn all_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Get all relationships.
    pub fn all_relationships(&self) -> impl Iterator<Item = &Relationship> {
        self.relationships.values()
    }

    /// Full-text search across entity names and descriptions.
    pub fn search(&self, query: &str) -> Vec<&Entity> {
        let query_lower = query.to_lowercase();
        self.entities
            .values()
            .filter(|e| {
                e.name.to_lowercase().contains(&query_lower)
                    || e.description.to_lowercase().contains(&query_lower)
                    || e.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Start building a query.
    pub fn query(&self) -> QueryBuilder<'_> {
        QueryBuilder::new(self)
    }

    // -----------------------------------------------------------------------
    // Graph traversal
    // -----------------------------------------------------------------------

    /// Get all entities directly connected to a given entity.
    pub fn neighbors(&self, entity: EntityId) -> Vec<(EntityId, &Relationship)> {
        self.relationships_of(entity)
            .into_iter()
            .map(|rel| {
                let other = if rel.source == entity {
                    rel.target
                } else {
                    rel.source
                };
                (other, rel)
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }

    /// Count entities by kind.
    pub fn entity_counts_by_kind(&self) -> HashMap<EntityKind, usize> {
        self.by_kind
            .iter()
            .map(|(k, ids)| (k.clone(), ids.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relationship::RelationshipKind;

    fn test_world() -> World {
        World::new(WorldMeta::new("Test World"))
    }

    #[test]
    fn add_and_get_entity() {
        let mut world = test_world();
        let entity = Entity::new(EntityKind::Character, "Kael Stormborn");
        let id = world.add_entity(entity).unwrap();
        let retrieved = world.get_entity(id).unwrap();
        assert_eq!(retrieved.name, "Kael Stormborn");
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut world = test_world();
        world
            .add_entity(Entity::new(EntityKind::Character, "Kael"))
            .unwrap();
        let result = world.add_entity(Entity::new(EntityKind::Location, "Kael"));
        assert!(result.is_err());
    }

    #[test]
    fn find_by_name_case_insensitive() {
        let mut world = test_world();
        world
            .add_entity(Entity::new(EntityKind::Character, "Kael Stormborn"))
            .unwrap();
        assert!(world.find_by_name("kael stormborn").is_some());
        assert!(world.find_by_name("KAEL STORMBORN").is_some());
        assert!(world.find_by_name("nobody").is_none());
    }

    #[test]
    fn add_and_query_relationships() {
        let mut world = test_world();
        let kael_id = world
            .add_entity(Entity::new(EntityKind::Character, "Kael"))
            .unwrap();
        let citadel_id = world
            .add_entity(Entity::new(EntityKind::Location, "The Iron Citadel"))
            .unwrap();

        let rel = Relationship::new(kael_id, RelationshipKind::LocatedAt, citadel_id);
        world.add_relationship(rel).unwrap();

        let rels = world.relationships_from(kael_id);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].target, citadel_id);

        let rels = world.relationships_to(citadel_id);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].source, kael_id);
    }

    #[test]
    fn bidirectional_relationship() {
        let mut world = test_world();
        let a = world
            .add_entity(Entity::new(EntityKind::Character, "Alice"))
            .unwrap();
        let b = world
            .add_entity(Entity::new(EntityKind::Character, "Bob"))
            .unwrap();

        let rel = Relationship::new(a, RelationshipKind::AlliedWith, b);
        world.add_relationship(rel).unwrap();

        // Both directions should show up
        assert_eq!(world.relationships_from(a).len(), 1);
        assert_eq!(world.relationships_from(b).len(), 1);
        assert_eq!(world.neighbors(a).len(), 1);
        assert_eq!(world.neighbors(b).len(), 1);
    }

    #[test]
    fn remove_entity_removes_relationships() {
        let mut world = test_world();
        let a = world
            .add_entity(Entity::new(EntityKind::Character, "Alice"))
            .unwrap();
        let b = world
            .add_entity(Entity::new(EntityKind::Character, "Bob"))
            .unwrap();

        world
            .add_relationship(Relationship::new(a, RelationshipKind::AlliedWith, b))
            .unwrap();

        world.remove_entity(a).unwrap();
        assert_eq!(world.relationship_count(), 0);
        assert!(world.relationships_of(b).is_empty());
    }

    #[test]
    fn search_finds_by_name_and_description() {
        let mut world = test_world();
        let mut e = Entity::new(EntityKind::Character, "Kael Stormborn");
        e.description = "A knight of the Order".to_string();
        world.add_entity(e).unwrap();

        assert_eq!(world.search("kael").len(), 1);
        assert_eq!(world.search("knight").len(), 1);
        assert_eq!(world.search("nobody").len(), 0);
    }

    #[test]
    fn entities_by_kind() {
        let mut world = test_world();
        world
            .add_entity(Entity::new(EntityKind::Character, "Kael"))
            .unwrap();
        world
            .add_entity(Entity::new(EntityKind::Character, "Elara"))
            .unwrap();
        world
            .add_entity(Entity::new(EntityKind::Location, "Citadel"))
            .unwrap();

        assert_eq!(world.entities_by_kind(&EntityKind::Character).len(), 2);
        assert_eq!(world.entities_by_kind(&EntityKind::Location).len(), 1);
        assert_eq!(world.entities_by_kind(&EntityKind::Faction).len(), 0);
    }
}
