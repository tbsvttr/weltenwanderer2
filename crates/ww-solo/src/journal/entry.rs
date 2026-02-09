//! Journal entry types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single entry in the session journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JournalEntry {
    /// An oracle yes/no query and its result.
    OracleQuery {
        /// The question asked.
        question: String,
        /// The likelihood level used.
        likelihood: String,
        /// Chaos factor at time of query.
        chaos: u32,
        /// The oracle's answer.
        result: String,
        /// Random event description, if triggered.
        random_event: Option<String>,
        /// When the query was made.
        timestamp: DateTime<Utc>,
    },
    /// The start of a new scene.
    SceneStart {
        /// Scene number.
        scene_number: u32,
        /// Expected scene setup.
        setup: String,
        /// Scene status after chaos check.
        status: String,
        /// When the scene started.
        timestamp: DateTime<Utc>,
    },
    /// The end of a scene.
    SceneEnd {
        /// Scene number.
        scene_number: u32,
        /// Player-written summary.
        summary: String,
        /// Chaos adjustment (+1, -1, or 0).
        chaos_adjustment: i32,
        /// When the scene ended.
        timestamp: DateTime<Utc>,
    },
    /// A narrative beat recorded by the player.
    NarrativeBeat {
        /// The narrative text.
        text: String,
        /// When recorded.
        timestamp: DateTime<Utc>,
    },
    /// A player note.
    Note {
        /// The note text.
        text: String,
        /// When recorded.
        timestamp: DateTime<Utc>,
    },
    /// An NPC reaction roll result.
    NpcReaction {
        /// NPC name.
        npc_name: String,
        /// Reaction result.
        reaction: String,
        /// The 2d10 roll total.
        roll: u32,
        /// When rolled.
        timestamp: DateTime<Utc>,
    },
    /// A random event that was generated.
    RandomEvent {
        /// Event description.
        description: String,
        /// When generated.
        timestamp: DateTime<Utc>,
    },
    /// A mechanics check (attribute test using world's ruleset).
    MechanicsCheck {
        /// Attribute or skill checked.
        attribute: String,
        /// Dice expression that was rolled.
        dice: String,
        /// Individual die values.
        values: Vec<u32>,
        /// The resolved outcome (e.g., "Success", "Critical Failure").
        outcome: String,
        /// When rolled.
        timestamp: DateTime<Utc>,
    },
    /// A freeform dice roll (not tied to a check).
    DiceRoll {
        /// Dice expression (e.g., "2d6", "d100").
        expression: String,
        /// Individual die values.
        values: Vec<u32>,
        /// Sum of all dice.
        total: u32,
        /// When rolled.
        timestamp: DateTime<Utc>,
    },
}
