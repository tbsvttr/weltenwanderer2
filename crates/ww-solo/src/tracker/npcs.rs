//! NPC tracking for the solo session.

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use ww_core::component::{CharacterComponent, CharacterStatus};
use ww_core::{EntityKind, World};

/// A tracked NPC in the solo story.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedNpc {
    /// NPC name.
    pub name: String,
    /// Optional notes about this NPC.
    pub notes: Option<String>,
}

/// List of tracked NPCs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NpcList {
    npcs: Vec<TrackedNpc>,
}

impl NpcList {
    /// Create an empty NPC list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an NPC to the tracker.
    pub fn add(&mut self, name: impl Into<String>) {
        self.npcs.push(TrackedNpc {
            name: name.into(),
            notes: None,
        });
    }

    /// Add an NPC with notes.
    pub fn add_with_notes(&mut self, name: impl Into<String>, notes: impl Into<String>) {
        self.npcs.push(TrackedNpc {
            name: name.into(),
            notes: Some(notes.into()),
        });
    }

    /// Remove an NPC by name. Returns true if found.
    pub fn remove(&mut self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        let len_before = self.npcs.len();
        self.npcs.retain(|n| n.name.to_lowercase() != name_lower);
        self.npcs.len() < len_before
    }

    /// Pick a random NPC.
    pub fn random(&self, rng: &mut StdRng) -> Option<&TrackedNpc> {
        if self.npcs.is_empty() {
            return None;
        }
        Some(&self.npcs[rng.random_range(0..self.npcs.len())])
    }

    /// Get all tracked NPCs.
    pub fn list(&self) -> &[TrackedNpc] {
        &self.npcs
    }

    /// Create an NPC list pre-populated from world characters.
    ///
    /// All characters except those with `CharacterStatus::Dead` are included.
    /// Notes are generated from species and occupation when available.
    pub fn from_world(world: &World) -> Self {
        let mut list = Self::new();
        for entity in world.entities_by_kind(&EntityKind::Character) {
            if let Some(ref char_comp) = entity.components.character {
                if char_comp.status == CharacterStatus::Dead {
                    continue;
                }
                let notes = build_character_notes(char_comp);
                if notes.is_empty() {
                    list.add(&entity.name);
                } else {
                    list.add_with_notes(&entity.name, notes);
                }
            } else {
                list.add(&entity.name);
            }
        }
        list
    }

    /// Number of tracked NPCs.
    pub fn count(&self) -> usize {
        self.npcs.len()
    }
}

/// Build a compact note string from character component data.
fn build_character_notes(comp: &CharacterComponent) -> String {
    let mut parts = Vec::new();
    if let Some(ref species) = comp.species {
        parts.push(species.clone());
    }
    if let Some(ref occupation) = comp.occupation {
        parts.push(occupation.clone());
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use ww_core::{Entity, World, WorldMeta};

    #[test]
    fn add_and_list() {
        let mut nl = NpcList::new();
        nl.add("Guard Captain");
        nl.add("Merchant");
        assert_eq!(nl.count(), 2);
        assert_eq!(nl.list()[0].name, "Guard Captain");
    }

    #[test]
    fn add_with_notes() {
        let mut nl = NpcList::new();
        nl.add_with_notes("Captain", "Friendly, knows about dungeon");
        assert_eq!(
            nl.list()[0].notes.as_deref(),
            Some("Friendly, knows about dungeon")
        );
    }

    #[test]
    fn remove_npc() {
        let mut nl = NpcList::new();
        nl.add("Guard Captain");
        nl.add("Merchant");
        assert!(nl.remove("Guard Captain"));
        assert_eq!(nl.count(), 1);
        assert_eq!(nl.list()[0].name, "Merchant");
    }

    #[test]
    fn remove_case_insensitive() {
        let mut nl = NpcList::new();
        nl.add("Guard Captain");
        assert!(nl.remove("guard captain"));
        assert_eq!(nl.count(), 0);
    }

    #[test]
    fn remove_nonexistent() {
        let mut nl = NpcList::new();
        nl.add("Guard");
        assert!(!nl.remove("Ghost"));
    }

    #[test]
    fn random_npc() {
        let mut nl = NpcList::new();
        nl.add("A");
        nl.add("B");
        let mut rng = StdRng::seed_from_u64(42);
        let n = nl.random(&mut rng).unwrap();
        assert!(n.name == "A" || n.name == "B");
    }

    #[test]
    fn random_empty() {
        let nl = NpcList::new();
        let mut rng = StdRng::seed_from_u64(42);
        assert!(nl.random(&mut rng).is_none());
    }

    #[test]
    fn serde_roundtrip() {
        let mut nl = NpcList::new();
        nl.add_with_notes("Guard", "Hostile");
        let json = serde_json::to_string(&nl).unwrap();
        let nl2: NpcList = serde_json::from_str(&json).unwrap();
        assert_eq!(nl2.count(), 1);
        assert_eq!(nl2.list()[0].notes.as_deref(), Some("Hostile"));
    }

    #[test]
    fn from_world_populates_npcs() {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut ch = Entity::new(EntityKind::Character, "Guard Captain");
        ch.components.character = Some(ww_core::component::CharacterComponent {
            species: Some("Human".to_string()),
            occupation: Some("Guard".to_string()),
            ..Default::default()
        });
        world.add_entity(ch).unwrap();

        let nl = NpcList::from_world(&world);
        assert_eq!(nl.count(), 1);
        assert_eq!(nl.list()[0].name, "Guard Captain");
        assert_eq!(nl.list()[0].notes.as_deref(), Some("Human, Guard"));
    }

    #[test]
    fn from_world_skips_dead() {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut alive = Entity::new(EntityKind::Character, "Alive NPC");
        alive.components.character = Some(ww_core::component::CharacterComponent {
            status: CharacterStatus::Alive,
            ..Default::default()
        });
        world.add_entity(alive).unwrap();

        let mut dead = Entity::new(EntityKind::Character, "Dead NPC");
        dead.components.character = Some(ww_core::component::CharacterComponent {
            status: CharacterStatus::Dead,
            ..Default::default()
        });
        world.add_entity(dead).unwrap();

        let nl = NpcList::from_world(&world);
        assert_eq!(nl.count(), 1);
        assert_eq!(nl.list()[0].name, "Alive NPC");
    }

    #[test]
    fn from_world_empty_world() {
        let world = World::new(WorldMeta::new("Empty"));
        let nl = NpcList::from_world(&world);
        assert_eq!(nl.count(), 0);
    }
}
