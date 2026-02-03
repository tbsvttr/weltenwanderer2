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

    // -----------------------------------------------------------------------
    // Large-world stress tests
    // -----------------------------------------------------------------------

    const ENTITY_KINDS: &[EntityKind] = &[
        EntityKind::Character,
        EntityKind::Location,
        EntityKind::Faction,
        EntityKind::Event,
        EntityKind::Item,
        EntityKind::Lore,
    ];

    /// Build a world with `n` entities spread across all built-in kinds.
    fn large_world(n: usize) -> (World, Vec<EntityId>) {
        let mut world = test_world();
        let mut ids = Vec::with_capacity(n);
        for i in 0..n {
            let kind = ENTITY_KINDS[i % ENTITY_KINDS.len()].clone();
            let mut entity = Entity::new(kind, format!("Entity {i}"));
            entity.description = format!("Description for entity number {i}");
            entity.tags = vec![format!("tag-{}", i % 10)];
            ids.push(world.add_entity(entity).unwrap());
        }
        (world, ids)
    }

    #[test]
    fn stress_1000_entities() {
        let (world, ids) = large_world(1_000);

        assert_eq!(world.entity_count(), 1_000);
        assert_eq!(ids.len(), 1_000);

        // Every entity is retrievable by ID
        for &id in &ids {
            assert!(world.get_entity(id).is_some());
        }

        // Name lookup works for all
        for i in 0..1_000 {
            assert!(
                world.find_by_name(&format!("Entity {i}")).is_some(),
                "entity {i} not found by name"
            );
        }

        // Kind index is correct
        let counts = world.entity_counts_by_kind();
        let expected_per_kind = 1_000 / ENTITY_KINDS.len();
        for kind in ENTITY_KINDS {
            let count = counts.get(kind).copied().unwrap_or(0);
            // With 1000 entities and 6 kinds: 166 or 167 each
            assert!(
                count >= expected_per_kind && count <= expected_per_kind + 1,
                "{kind}: expected ~{expected_per_kind}, got {count}"
            );
        }
    }

    #[test]
    fn stress_dense_relationships() {
        let (mut world, ids) = large_world(200);

        // Create a relationship from every entity to the next (ring topology)
        let rel_kinds = [
            RelationshipKind::LocatedAt,
            RelationshipKind::MemberOf,
            RelationshipKind::AlliedWith,
            RelationshipKind::OwnedBy,
        ];
        for i in 0..ids.len() {
            let next = (i + 1) % ids.len();
            let kind = rel_kinds[i % rel_kinds.len()].clone();
            world
                .add_relationship(Relationship::new(ids[i], kind, ids[next]))
                .unwrap();
        }

        assert_eq!(world.relationship_count(), 200);

        // Every entity should have at least one outgoing relationship
        for &id in &ids {
            assert!(
                !world.relationships_from(id).is_empty(),
                "entity {id} has no outgoing relationships"
            );
        }

        // Every entity should have at least one incoming relationship
        for &id in &ids {
            assert!(
                !world.relationships_to(id).is_empty(),
                "entity {id} has no incoming relationships"
            );
        }

        // Neighbors should include at least the ring neighbors
        for &id in &ids {
            assert!(
                !world.neighbors(id).is_empty(),
                "entity {id} has no neighbors"
            );
        }
    }

    #[test]
    fn stress_hub_entity_many_connections() {
        let (mut world, ids) = large_world(500);

        // Make entity 0 a hub connected to all others
        let hub = ids[0];
        for &id in &ids[1..] {
            world
                .add_relationship(Relationship::new(hub, RelationshipKind::LeaderOf, id))
                .unwrap();
        }

        assert_eq!(world.relationship_count(), 499);
        assert_eq!(world.relationships_from(hub).len(), 499);
        assert_eq!(world.neighbors(hub).len(), 499);
    }

    #[test]
    fn stress_search_large_world() {
        let (world, _) = large_world(1_000);

        // Exact match by name substring
        let results = world.search("Entity 42");
        assert!(
            results.iter().any(|e| e.name == "Entity 42"),
            "search should find Entity 42"
        );

        // Search by description content
        let results = world.search("entity number 999");
        assert!(
            results.iter().any(|e| e.name == "Entity 999"),
            "search should find entity by description"
        );

        // Search by tag
        let results = world.search("tag-0");
        assert_eq!(results.len(), 100, "100 entities should have tag-0");
    }

    #[test]
    fn stress_query_builder_large_world() {
        let (world, _) = large_world(1_000);

        // Filter by kind
        let chars = world.query().kind(EntityKind::Character).execute();
        assert!(chars.len() >= 166);

        // Filter by tag
        let tagged = world.query().tag("tag-0").execute();
        assert_eq!(tagged.len(), 100);

        // Limit + offset for pagination
        let page1 = world.query().limit(10).execute();
        let page2 = world.query().offset(10).limit(10).execute();
        assert_eq!(page1.len(), 10);
        assert_eq!(page2.len(), 10);
        // Pages should not overlap
        for e in &page1 {
            assert!(
                !page2.iter().any(|p| p.id == e.id),
                "pages should not overlap"
            );
        }

        // Count should match
        let total = world.query().count();
        assert_eq!(total, 1_000);
    }

    #[test]
    fn stress_remove_entity_with_many_relationships() {
        let (mut world, ids) = large_world(100);

        // Connect entity 0 to all others
        let hub = ids[0];
        for &id in &ids[1..] {
            world
                .add_relationship(Relationship::new(hub, RelationshipKind::MemberOf, id))
                .unwrap();
        }
        assert_eq!(world.relationship_count(), 99);

        // Remove the hub — all 99 relationships should cascade
        world.remove_entity(hub).unwrap();
        assert_eq!(world.entity_count(), 99);
        assert_eq!(world.relationship_count(), 0);

        // No dangling references
        for &id in &ids[1..] {
            assert!(world.relationships_of(id).is_empty());
        }
    }

    #[test]
    fn stress_bidirectional_at_scale() {
        let (mut world, ids) = large_world(100);

        // AlliedWith is bidirectional — create chain
        for i in 0..ids.len() - 1 {
            world
                .add_relationship(Relationship::new(
                    ids[i],
                    RelationshipKind::AlliedWith,
                    ids[i + 1],
                ))
                .unwrap();
        }

        assert_eq!(world.relationship_count(), 99);

        // Interior nodes should have 2 neighbors (left + right)
        for &id in &ids[1..ids.len() - 1] {
            assert_eq!(
                world.neighbors(id).len(),
                2,
                "interior node should have 2 neighbors"
            );
        }
        // Endpoints should have 1 neighbor
        assert_eq!(world.neighbors(ids[0]).len(), 1);
        assert_eq!(world.neighbors(ids[ids.len() - 1]).len(), 1);
    }
}
