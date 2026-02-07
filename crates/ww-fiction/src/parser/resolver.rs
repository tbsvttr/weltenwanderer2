//! Entity name resolution with fuzzy matching.

use strsim::jaro_winkler;
use ww_core::{EntityId, RelationshipKind, World};

/// Minimum similarity score for fuzzy matching (0.0-1.0).
const FUZZY_THRESHOLD: f64 = 0.8;

/// Resolve an entity name to an ID using exact or fuzzy matching.
pub fn resolve_entity(world: &World, input: &str) -> Option<EntityId> {
    // Try exact match first (case-insensitive)
    if let Some(entity) = world.find_by_name(input) {
        return Some(entity.id);
    }

    // Try fuzzy matching
    let candidates = fuzzy_match(world, input, FUZZY_THRESHOLD);
    candidates.first().map(|(id, _)| *id)
}

/// Find entities matching the input with a similarity score above the threshold.
///
/// Returns a list of (EntityId, score) sorted by score descending.
pub fn fuzzy_match(world: &World, input: &str, threshold: f64) -> Vec<(EntityId, f64)> {
    let input_lower = input.to_lowercase();
    let mut matches: Vec<(EntityId, f64)> = world
        .all_entities()
        .filter_map(|entity| {
            let name_lower = entity.name.to_lowercase();
            let score = jaro_winkler(&input_lower, &name_lower);
            if score >= threshold {
                Some((entity.id, score))
            } else {
                None
            }
        })
        .collect();

    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    matches
}

/// Suggest entity names that start with or contain the partial input.
pub fn suggest_entities(world: &World, partial: &str, limit: usize) -> Vec<String> {
    let partial_lower = partial.to_lowercase();
    let mut suggestions: Vec<(String, f64)> = world
        .all_entities()
        .filter_map(|entity| {
            let name_lower = entity.name.to_lowercase();
            if name_lower.starts_with(&partial_lower) {
                Some((entity.name.clone(), 2.0))
            } else if name_lower.contains(&partial_lower) {
                Some((entity.name.clone(), 1.0))
            } else {
                let score = jaro_winkler(&partial_lower, &name_lower);
                if score >= 0.6 {
                    Some((entity.name.clone(), score))
                } else {
                    None
                }
            }
        })
        .collect();

    suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    suggestions
        .into_iter()
        .take(limit)
        .map(|(name, _)| name)
        .collect()
}

/// Resolve an entity at a specific location.
pub fn resolve_entity_at_location(
    world: &World,
    input: &str,
    location: EntityId,
) -> Option<EntityId> {
    let input_lower = input.to_lowercase();

    // Get all entities at this location
    let at_location: Vec<_> = world
        .all_entities()
        .filter(|e| {
            world.relationships_from(e.id).iter().any(|r| {
                r.target == location
                    && matches!(
                        r.kind,
                        RelationshipKind::LocatedAt
                            | RelationshipKind::BasedAt
                            | RelationshipKind::ContainedIn
                    )
            })
        })
        .collect();

    // Try exact match first
    for entity in &at_location {
        if entity.name.to_lowercase() == input_lower {
            return Some(entity.id);
        }
    }

    // Try fuzzy match
    let mut best: Option<(EntityId, f64)> = None;
    for entity in &at_location {
        let score = jaro_winkler(&input_lower, &entity.name.to_lowercase());
        if score >= FUZZY_THRESHOLD && (best.is_none() || score > best.unwrap().1) {
            best = Some((entity.id, score));
        }
    }

    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::{Entity, EntityKind, WorldMeta};

    fn test_world() -> World {
        let mut world = World::new(WorldMeta::new("Test"));
        world
            .add_entity(Entity::new(EntityKind::Character, "Kael Stormborn"))
            .unwrap();
        world
            .add_entity(Entity::new(EntityKind::Character, "Elara Nightwhisper"))
            .unwrap();
        world
            .add_entity(Entity::new(EntityKind::Location, "the Iron Citadel"))
            .unwrap();
        world
            .add_entity(Entity::new(EntityKind::Item, "Rusty Sword"))
            .unwrap();
        world
    }

    #[test]
    fn exact_match() {
        let world = test_world();
        let id = resolve_entity(&world, "Kael Stormborn");
        assert!(id.is_some());
        let entity = world.get_entity(id.unwrap()).unwrap();
        assert_eq!(entity.name, "Kael Stormborn");
    }

    #[test]
    fn case_insensitive_match() {
        let world = test_world();
        let id = resolve_entity(&world, "kael stormborn");
        assert!(id.is_some());
    }

    #[test]
    fn fuzzy_match_typo() {
        let world = test_world();
        let id = resolve_entity(&world, "Kael Stormbon");
        assert!(id.is_some());
        let entity = world.get_entity(id.unwrap()).unwrap();
        assert_eq!(entity.name, "Kael Stormborn");
    }

    #[test]
    fn no_match() {
        let world = test_world();
        let id = resolve_entity(&world, "completely different");
        assert!(id.is_none());
    }

    #[test]
    fn suggest_prefix() {
        let world = test_world();
        let suggestions = suggest_entities(&world, "Ka", 5);
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].starts_with("Ka"));
    }

    #[test]
    fn suggest_substring() {
        let world = test_world();
        let suggestions = suggest_entities(&world, "Storm", 5);
        assert!(suggestions.iter().any(|s| s.contains("Storm")));
    }
}
