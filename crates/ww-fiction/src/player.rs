//! Player state management.

use std::collections::HashMap;
use ww_core::EntityId;
use ww_core::entity::MetadataValue;

/// The player's current state in the fiction session.
#[derive(Debug, Clone)]
pub struct PlayerState {
    /// The player's character entity ID.
    pub entity_id: EntityId,
    /// Current location entity ID.
    pub location: EntityId,
    /// Items in the player's inventory.
    pub inventory: Vec<EntityId>,
    /// Knowledge flags (discovered facts).
    pub knowledge: HashMap<String, bool>,
    /// Arbitrary state flags.
    pub flags: HashMap<String, MetadataValue>,
}

impl PlayerState {
    /// Create a new player state at the given location.
    pub fn new(entity_id: EntityId, location: EntityId) -> Self {
        Self {
            entity_id,
            location,
            inventory: Vec::new(),
            knowledge: HashMap::new(),
            flags: HashMap::new(),
        }
    }

    /// Check if the player has a knowledge flag set.
    pub fn has_knowledge(&self, key: &str) -> bool {
        self.knowledge.get(key).copied().unwrap_or(false)
    }

    /// Set a knowledge flag.
    pub fn set_knowledge(&mut self, key: impl Into<String>, value: bool) {
        self.knowledge.insert(key.into(), value);
    }

    /// Check if the player has a specific flag value.
    pub fn has_flag(&self, key: &str) -> bool {
        self.flags.contains_key(key)
    }

    /// Get a flag value.
    pub fn get_flag(&self, key: &str) -> Option<&MetadataValue> {
        self.flags.get(key)
    }

    /// Set a flag value.
    pub fn set_flag(&mut self, key: impl Into<String>, value: MetadataValue) {
        self.flags.insert(key.into(), value);
    }

    /// Check if the player has an item.
    pub fn has_item(&self, item_id: EntityId) -> bool {
        self.inventory.contains(&item_id)
    }

    /// Add an item to inventory.
    pub fn add_item(&mut self, item_id: EntityId) {
        if !self.inventory.contains(&item_id) {
            self.inventory.push(item_id);
        }
    }

    /// Remove an item from inventory.
    pub fn remove_item(&mut self, item_id: EntityId) -> bool {
        if let Some(pos) = self.inventory.iter().position(|&id| id == item_id) {
            self.inventory.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::EntityId;

    #[test]
    fn player_state_new() {
        let player_id = EntityId::new();
        let location_id = EntityId::new();
        let state = PlayerState::new(player_id, location_id);

        assert_eq!(state.entity_id, player_id);
        assert_eq!(state.location, location_id);
        assert!(state.inventory.is_empty());
        assert!(state.knowledge.is_empty());
        assert!(state.flags.is_empty());
    }

    #[test]
    fn knowledge_flags() {
        let mut state = PlayerState::new(EntityId::new(), EntityId::new());

        assert!(!state.has_knowledge("secret"));
        state.set_knowledge("secret", true);
        assert!(state.has_knowledge("secret"));
        state.set_knowledge("secret", false);
        assert!(!state.has_knowledge("secret"));
    }

    #[test]
    fn inventory_management() {
        let mut state = PlayerState::new(EntityId::new(), EntityId::new());
        let item = EntityId::new();

        assert!(!state.has_item(item));
        state.add_item(item);
        assert!(state.has_item(item));

        // Adding again should not duplicate
        state.add_item(item);
        assert_eq!(state.inventory.len(), 1);

        assert!(state.remove_item(item));
        assert!(!state.has_item(item));

        // Removing again should return false
        assert!(!state.remove_item(item));
    }

    #[test]
    fn flag_values() {
        let mut state = PlayerState::new(EntityId::new(), EntityId::new());

        assert!(!state.has_flag("reputation"));
        state.set_flag("reputation", MetadataValue::Integer(50));
        assert!(state.has_flag("reputation"));
        assert_eq!(
            state.get_flag("reputation"),
            Some(&MetadataValue::Integer(50))
        );
    }
}
