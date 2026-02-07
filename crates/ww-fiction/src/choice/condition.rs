//! Condition evaluation for dialogue branches.

use ww_core::entity::MetadataValue;
use ww_core::{EntityKind, RelationshipKind, World};

use crate::player::PlayerState;

/// A condition that can be evaluated against world/player state.
#[derive(Debug, Clone, Default)]
pub enum Condition {
    /// Check if player is at a specific location.
    PlayerAt {
        /// Location name.
        location: String,
    },
    /// Check if player has an item.
    HasItem {
        /// Item name.
        item: String,
    },
    /// Check if a knowledge flag is set.
    HasKnowledge {
        /// Knowledge key.
        key: String,
    },
    /// Check if a flag equals a value.
    FlagEquals {
        /// Flag key.
        key: String,
        /// Expected value.
        value: MetadataValue,
    },
    /// Check if a relationship exists.
    RelationshipExists {
        /// Source entity name.
        from: String,
        /// Relationship kind.
        kind: RelationshipKind,
        /// Target entity name.
        to: String,
    },
    /// Check if an entity has a specific status.
    EntityStatus {
        /// Entity name.
        entity: String,
        /// Expected status.
        status: String,
    },
    /// Logical NOT.
    Not(Box<Condition>),
    /// Logical AND.
    And(Vec<Condition>),
    /// Logical OR.
    Or(Vec<Condition>),
    /// Always true.
    #[default]
    Always,
}

impl Condition {
    /// Evaluate the condition against the current state.
    pub fn evaluate(&self, world: &World, player: &PlayerState) -> bool {
        match self {
            Condition::PlayerAt { location } => {
                if let Some(loc) = world.find_by_name(location) {
                    player.location == loc.id
                } else {
                    false
                }
            }
            Condition::HasItem { item } => {
                if let Some(item_entity) = world.find_by_name(item) {
                    player.has_item(item_entity.id)
                } else {
                    false
                }
            }
            Condition::HasKnowledge { key } => player.has_knowledge(key),
            Condition::FlagEquals { key, value } => {
                player.get_flag(key).is_some_and(|v| v == value)
            }
            Condition::RelationshipExists { from, kind, to } => {
                let from_entity = world.find_by_name(from);
                let to_entity = world.find_by_name(to);
                if let (Some(f), Some(t)) = (from_entity, to_entity) {
                    world
                        .relationships_from(f.id)
                        .iter()
                        .any(|r| r.target == t.id && &r.kind == kind)
                } else {
                    false
                }
            }
            Condition::EntityStatus { entity, status } => {
                if let Some(e) = world.find_by_name(entity)
                    && e.kind == EntityKind::Character
                    && let Some(char_comp) = &e.components.character
                {
                    let status_str = format!("{:?}", char_comp.status);
                    return status_str.eq_ignore_ascii_case(status);
                }
                false
            }
            Condition::Not(inner) => !inner.evaluate(world, player),
            Condition::And(conditions) => conditions.iter().all(|c| c.evaluate(world, player)),
            Condition::Or(conditions) => conditions.iter().any(|c| c.evaluate(world, player)),
            Condition::Always => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::{Entity, EntityId, WorldMeta};

    fn test_world() -> World {
        let mut world = World::new(WorldMeta::new("Test"));
        world
            .add_entity(Entity::new(EntityKind::Location, "the Tavern"))
            .unwrap();
        world
            .add_entity(Entity::new(EntityKind::Item, "Golden Key"))
            .unwrap();
        world
    }

    #[test]
    fn player_at_location() {
        let world = test_world();
        let tavern = world.find_by_name("the Tavern").unwrap();
        let player = PlayerState::new(EntityId::new(), tavern.id);

        let cond = Condition::PlayerAt {
            location: "the Tavern".to_string(),
        };
        assert!(cond.evaluate(&world, &player));

        let cond = Condition::PlayerAt {
            location: "somewhere else".to_string(),
        };
        assert!(!cond.evaluate(&world, &player));
    }

    #[test]
    fn has_item() {
        let world = test_world();
        let tavern = world.find_by_name("the Tavern").unwrap();
        let key = world.find_by_name("Golden Key").unwrap();
        let mut player = PlayerState::new(EntityId::new(), tavern.id);

        let cond = Condition::HasItem {
            item: "Golden Key".to_string(),
        };
        assert!(!cond.evaluate(&world, &player));

        player.add_item(key.id);
        assert!(cond.evaluate(&world, &player));
    }

    #[test]
    fn has_knowledge() {
        let world = test_world();
        let tavern = world.find_by_name("the Tavern").unwrap();
        let mut player = PlayerState::new(EntityId::new(), tavern.id);

        let cond = Condition::HasKnowledge {
            key: "secret".to_string(),
        };
        assert!(!cond.evaluate(&world, &player));

        player.set_knowledge("secret", true);
        assert!(cond.evaluate(&world, &player));
    }

    #[test]
    fn logical_not() {
        let world = test_world();
        let tavern = world.find_by_name("the Tavern").unwrap();
        let player = PlayerState::new(EntityId::new(), tavern.id);

        let cond = Condition::Not(Box::new(Condition::HasKnowledge {
            key: "secret".to_string(),
        }));
        assert!(cond.evaluate(&world, &player));
    }

    #[test]
    fn logical_and() {
        let world = test_world();
        let tavern = world.find_by_name("the Tavern").unwrap();
        let mut player = PlayerState::new(EntityId::new(), tavern.id);
        player.set_knowledge("a", true);
        player.set_knowledge("b", true);

        let cond = Condition::And(vec![
            Condition::HasKnowledge {
                key: "a".to_string(),
            },
            Condition::HasKnowledge {
                key: "b".to_string(),
            },
        ]);
        assert!(cond.evaluate(&world, &player));

        player.set_knowledge("b", false);
        assert!(!cond.evaluate(&world, &player));
    }

    #[test]
    fn logical_or() {
        let world = test_world();
        let tavern = world.find_by_name("the Tavern").unwrap();
        let mut player = PlayerState::new(EntityId::new(), tavern.id);
        player.set_knowledge("a", true);

        let cond = Condition::Or(vec![
            Condition::HasKnowledge {
                key: "a".to_string(),
            },
            Condition::HasKnowledge {
                key: "b".to_string(),
            },
        ]);
        assert!(cond.evaluate(&world, &player));
    }
}
