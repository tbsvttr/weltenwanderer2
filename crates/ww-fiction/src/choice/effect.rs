//! Effects that modify world or player state.

use ww_core::RelationshipKind;
use ww_core::entity::MetadataValue;

/// An effect that can be applied when a choice is made.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Set a player flag.
    SetFlag {
        /// Flag key.
        key: String,
        /// Value to set.
        value: MetadataValue,
    },
    /// Set a knowledge flag.
    SetKnowledge {
        /// Knowledge key.
        key: String,
        /// Value (true/false).
        value: bool,
    },
    /// Give an item to the player.
    GiveItem {
        /// Item name.
        item: String,
    },
    /// Take an item from the player.
    TakeItem {
        /// Item name.
        item: String,
    },
    /// Move the player to a location.
    MovePlayer {
        /// Target location name.
        location: String,
    },
    /// Create a relationship between entities.
    CreateRelationship {
        /// Source entity name.
        from: String,
        /// Relationship kind.
        kind: RelationshipKind,
        /// Target entity name.
        to: String,
    },
    /// Remove a relationship between entities.
    RemoveRelationship {
        /// Source entity name.
        from: String,
        /// Relationship kind.
        kind: RelationshipKind,
        /// Target entity name.
        to: String,
    },
    /// Emit a dialogue (branch to another dialogue).
    EmitDialogue {
        /// Dialogue ID to show.
        dialogue_id: String,
    },
}
