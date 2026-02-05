use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entity::{EntityId, MetadataValue};

/// The set of typed components attached to an entity.
/// Each entity kind has an associated component, but entities can hold any combination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentSet {
    /// Location data for places, regions, and spatial entities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationComponent>,
    /// Character data for people, creatures, and NPCs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<CharacterComponent>,
    /// Faction data for organizations, guilds, and groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<FactionComponent>,
    /// Event data for historical or scheduled occurrences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventComponent>,
    /// Item data for objects, artifacts, and equipment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemComponent>,
    /// Lore data for myths, legends, and world knowledge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lore: Option<LoreComponent>,
    /// Simulation-specific data (schedule, needs, speed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulation: Option<SimulationComponent>,
}

// ---------------------------------------------------------------------------
// Location
// ---------------------------------------------------------------------------

/// Component describing a location in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationComponent {
    /// Subtype: "fortress", "city", "region", "room", etc.
    pub location_type: String,
    /// The entity ID of the parent location that contains this one.
    pub parent_location: Option<EntityId>,
    /// Climate description, e.g. "tropical", "arid", "temperate".
    pub climate: Option<String>,
    /// Terrain description, e.g. "mountain", "forest", "plains".
    pub terrain: Option<String>,
    /// Estimated population count of this location.
    pub population: Option<u64>,
    /// Spatial coordinates on the world map.
    pub coordinates: Option<Coordinates>,
}

impl Default for LocationComponent {
    fn default() -> Self {
        Self {
            location_type: "location".to_string(),
            parent_location: None,
            climate: None,
            terrain: None,
            population: None,
            coordinates: None,
        }
    }
}

/// A point in 2D or 3D world space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    /// Horizontal position on the world map.
    pub x: f64,
    /// Vertical position on the world map.
    pub y: f64,
    /// Optional elevation or depth component.
    pub z: Option<f64>,
}

// ---------------------------------------------------------------------------
// Character
// ---------------------------------------------------------------------------

/// Component describing a character such as a person, creature, or NPC.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterComponent {
    /// Species or race of the character, e.g. "human", "elf", "dragon".
    pub species: Option<String>,
    /// Primary occupation or role, e.g. "blacksmith", "knight".
    pub occupation: Option<String>,
    /// Whether the character is alive, dead, or in another state.
    pub status: CharacterStatus,
    /// Descriptive personality or physical traits.
    pub traits: Vec<String>,
    /// Numeric or freeform stats such as strength, intelligence, etc.
    pub stats: HashMap<String, MetadataValue>,
}

/// The living status of a character.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterStatus {
    /// The character is currently alive.
    Alive,
    /// The character is deceased.
    Dead,
    /// The character's status is not known.
    #[default]
    Unknown,
    /// A user-defined status value.
    Custom(String),
}

// ---------------------------------------------------------------------------
// Faction
// ---------------------------------------------------------------------------

/// Component describing a faction, organization, or group.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionComponent {
    /// Subtype of faction, e.g. "guild", "kingdom", "cult".
    pub faction_type: Option<String>,
    /// Moral or political alignment, e.g. "lawful good", "neutral".
    pub alignment: Option<String>,
    /// Core values or principles the faction upholds.
    pub values: Vec<String>,
    /// Named resources the faction controls or possesses.
    pub resources: HashMap<String, MetadataValue>,
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

/// Component describing a world event or historical occurrence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventComponent {
    /// Category of event, e.g. "battle", "festival", "natural disaster".
    pub event_type: Option<String>,
    /// When the event occurred in the world's calendar.
    pub date: Option<WorldDate>,
    /// How long the event lasted, e.g. "3 days", "a century".
    pub duration: Option<String>,
    /// The result or consequence of the event.
    pub outcome: Option<String>,
}

/// A date in the world's calendar system. Supports custom calendars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldDate {
    /// Year number; may be negative for dates before year zero.
    pub year: i64,
    /// Optional month within the year.
    pub month: Option<u32>,
    /// Optional day within the month.
    pub day: Option<u32>,
    /// Optional named era, e.g. "Before Sundering", "Third Age".
    pub era: Option<String>,
}

impl WorldDate {
    /// Creates a new `WorldDate` with only a year, leaving month, day, and era unset.
    pub fn new(year: i64) -> Self {
        Self {
            year,
            month: None,
            day: None,
            era: None,
        }
    }

    /// Returns a sort key for chronological ordering.
    pub fn sort_key(&self) -> i64 {
        let mut key = self.year * 10000;
        if let Some(m) = self.month {
            key += (m as i64) * 100;
        }
        if let Some(d) = self.day {
            key += d as i64;
        }
        key
    }
}

impl std::fmt::Display for WorldDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(era) = &self.era {
            write!(f, "{}", era)?;
            write!(f, " ")?;
        }
        write!(f, "Year {}", self.year)?;
        if let Some(m) = self.month {
            write!(f, ", Month {m}")?;
        }
        if let Some(d) = self.day {
            write!(f, ", Day {d}")?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Item
// ---------------------------------------------------------------------------

/// Component describing an item, artifact, or piece of equipment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemComponent {
    /// Category of item, e.g. "weapon", "potion", "artifact".
    pub item_type: Option<String>,
    /// Rarity tier, e.g. "common", "rare", "legendary".
    pub rarity: Option<String>,
    /// Arbitrary key-value properties such as weight, damage, or effects.
    pub properties: HashMap<String, MetadataValue>,
}

// ---------------------------------------------------------------------------
// Lore
// ---------------------------------------------------------------------------

/// Component describing a piece of world lore, myth, or legend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoreComponent {
    /// Category of lore, e.g. "myth", "prophecy", "historical record".
    pub lore_type: Option<String>,
    /// In-world source of the lore, e.g. "Elder Scrolls", "oral tradition".
    pub source: Option<String>,
    /// How trustworthy this lore is, e.g. "verified", "disputed", "legend".
    pub reliability: Option<String>,
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

/// A single schedule time slot for simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimScheduleEntry {
    /// Start hour (0.0-24.0).
    pub start_hour: f64,
    /// End hour (0.0-24.0); may be less than start for midnight wrap.
    pub end_hour: f64,
    /// Activity name (e.g., "eat", "work", "rest").
    pub activity: String,
}

/// Simulation-specific data for characters.
///
/// This component stores configuration that the simulation crate reads
/// to customize character behavior. All fields are optional; when absent,
/// the simulation uses default values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationComponent {
    /// Custom schedule entries; if None, uses default NPC schedule.
    pub schedule: Option<Vec<SimScheduleEntry>>,
    /// Initial need levels (0.0-1.0); if None, starts at 1.0.
    pub initial_needs: Option<HashMap<String, f64>>,
    /// Movement speed in edges per tick; if None, uses default 1.0.
    pub speed: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_date_sort_key() {
        let d1 = WorldDate {
            year: -1247,
            month: Some(3),
            day: Some(15),
            era: None,
        };
        let d2 = WorldDate {
            year: -1247,
            month: Some(6),
            day: None,
            era: None,
        };
        assert!(d1.sort_key() < d2.sort_key());
    }

    #[test]
    fn world_date_display() {
        let d = WorldDate {
            year: -1247,
            month: Some(3),
            day: Some(15),
            era: Some("Before Sundering".to_string()),
        };
        let s = d.to_string();
        assert!(s.contains("-1247"));
        assert!(s.contains("Month 3"));
        assert!(s.contains("Day 15"));
    }
}
