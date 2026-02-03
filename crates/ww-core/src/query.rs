use crate::entity::{Entity, EntityId, EntityKind};
use crate::world::World;

/// A builder for filtering and searching entities in a world.
pub struct QueryBuilder<'w> {
    world: &'w World,
    kind_filter: Option<EntityKind>,
    tag_filters: Vec<String>,
    name_contains: Option<String>,
    related_to: Option<EntityId>,
    has_property: Option<String>,
    limit: Option<usize>,
    offset: usize,
}

impl<'w> QueryBuilder<'w> {
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            kind_filter: None,
            tag_filters: Vec::new(),
            name_contains: None,
            related_to: None,
            has_property: None,
            limit: None,
            offset: 0,
        }
    }

    /// Filter by entity kind.
    pub fn kind(mut self, kind: EntityKind) -> Self {
        self.kind_filter = Some(kind);
        self
    }

    /// Filter to entities that have a specific tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag_filters.push(tag.into());
        self
    }

    /// Filter to entities whose name contains the given substring (case-insensitive).
    pub fn name_contains(mut self, s: impl Into<String>) -> Self {
        self.name_contains = Some(s.into().to_lowercase());
        self
    }

    /// Filter to entities that have a relationship with the given entity.
    pub fn related_to(mut self, id: EntityId) -> Self {
        self.related_to = Some(id);
        self
    }

    /// Filter to entities that have a specific property key.
    pub fn has_property(mut self, key: impl Into<String>) -> Self {
        self.has_property = Some(key.into());
        self
    }

    /// Limit the number of results.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Skip the first N results.
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = n;
        self
    }

    /// Execute the query and return matching entities.
    pub fn execute(self) -> Vec<&'w Entity> {
        let mut results: Vec<&Entity> = self
            .world
            .all_entities()
            .filter(|e| self.matches(e))
            .collect();

        // Sort by name for deterministic output
        results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        // Apply offset and limit
        let results: Vec<&Entity> = results.into_iter().skip(self.offset).collect();
        if let Some(limit) = self.limit {
            results.into_iter().take(limit).collect()
        } else {
            results
        }
    }

    /// Count matching entities without collecting them.
    pub fn count(self) -> usize {
        self.world
            .all_entities()
            .filter(|e| self.matches(e))
            .count()
    }

    fn matches(&self, entity: &Entity) -> bool {
        // Kind filter
        if let Some(ref kind) = self.kind_filter
            && entity.kind != *kind {
                return false;
            }

        // Tag filters (all must match)
        for tag in &self.tag_filters {
            let tag_lower = tag.to_lowercase();
            if !entity
                .tags
                .iter()
                .any(|t| t.to_lowercase() == tag_lower)
            {
                return false;
            }
        }

        // Name contains
        if let Some(ref s) = self.name_contains
            && !entity.name.to_lowercase().contains(s) {
                return false;
            }

        // Related to
        if let Some(related_id) = self.related_to {
            let neighbors: Vec<_> = self
                .world
                .neighbors(entity.id)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            if !neighbors.contains(&related_id) {
                return false;
            }
        }

        // Has property
        if let Some(ref key) = self.has_property
            && !entity.properties.contains_key(key) {
                return false;
            }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::MetadataValue;
    use crate::world::WorldMeta;

    fn test_world() -> World {
        let mut world = World::new(WorldMeta::new("Test"));

        let mut kael = Entity::new(EntityKind::Character, "Kael Stormborn");
        kael.tags = vec!["knight".to_string(), "protagonist".to_string()];
        kael.properties
            .insert("level".to_string(), MetadataValue::Integer(5));
        world.add_entity(kael).unwrap();

        let mut elara = Entity::new(EntityKind::Character, "Elara Nightwhisper");
        elara.tags = vec!["mage".to_string(), "protagonist".to_string()];
        world.add_entity(elara).unwrap();

        let citadel = Entity::new(EntityKind::Location, "The Iron Citadel");
        world.add_entity(citadel).unwrap();

        world
    }

    #[test]
    fn query_by_kind() {
        let world = test_world();
        let results = world.query().kind(EntityKind::Character).execute();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_by_tag() {
        let world = test_world();
        let results = world
            .query()
            .kind(EntityKind::Character)
            .tag("knight")
            .execute();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Kael Stormborn");
    }

    #[test]
    fn query_by_name_contains() {
        let world = test_world();
        let results = world.query().name_contains("iron").execute();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn query_with_limit_and_offset() {
        let world = test_world();
        let all = world.query().execute();
        assert_eq!(all.len(), 3);

        let limited = world.query().limit(2).execute();
        assert_eq!(limited.len(), 2);

        let offset = world.query().offset(1).limit(1).execute();
        assert_eq!(offset.len(), 1);
    }

    #[test]
    fn query_has_property() {
        let world = test_world();
        let results = world.query().has_property("level").execute();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Kael Stormborn");
    }

    #[test]
    fn query_count() {
        let world = test_world();
        assert_eq!(world.query().kind(EntityKind::Character).count(), 2);
    }
}
