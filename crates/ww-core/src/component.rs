use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entity::{EntityId, MetadataValue};

/// The set of typed components attached to an entity.
/// Each entity kind has an associated component, but entities can hold any combination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<CharacterComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<FactionComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lore: Option<LoreComponent>,
}

// ---------------------------------------------------------------------------
// Location
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationComponent {
    /// Subtype: "fortress", "city", "region", "room", etc.
    pub location_type: String,
    pub parent_location: Option<EntityId>,
    pub climate: Option<String>,
    pub terrain: Option<String>,
    pub population: Option<u64>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}

// ---------------------------------------------------------------------------
// Character
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterComponent {
    pub species: Option<String>,
    pub occupation: Option<String>,
    pub status: CharacterStatus,
    pub traits: Vec<String>,
    pub stats: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterStatus {
    Alive,
    Dead,
    #[default]
    Unknown,
    Custom(String),
}

// ---------------------------------------------------------------------------
// Faction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionComponent {
    pub faction_type: Option<String>,
    pub alignment: Option<String>,
    pub values: Vec<String>,
    pub resources: HashMap<String, MetadataValue>,
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventComponent {
    pub event_type: Option<String>,
    pub date: Option<WorldDate>,
    pub duration: Option<String>,
    pub outcome: Option<String>,
}

/// A date in the world's calendar system. Supports custom calendars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldDate {
    pub year: i64,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub era: Option<String>,
}

impl WorldDate {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemComponent {
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub properties: HashMap<String, MetadataValue>,
}

// ---------------------------------------------------------------------------
// Lore
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoreComponent {
    pub lore_type: Option<String>,
    pub source: Option<String>,
    pub reliability: Option<String>,
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
