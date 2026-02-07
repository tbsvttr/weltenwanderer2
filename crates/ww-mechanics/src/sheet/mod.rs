//! Character sheets with attributes, skills, focuses, and tracks.
//!
//! A character sheet is built from an entity's `mechanics.*` properties
//! and a ruleset that defines which attributes, skills, and tracks exist.

pub mod track;

pub use track::Track;

use std::collections::HashMap;

use ww_core::entity::{Entity, MetadataValue};

use crate::error::{MechError, MechResult};
use crate::rules::{RuleSet, TrackDefinition};

/// A character's mechanical state within a game system.
#[derive(Debug, Clone)]
pub struct CharacterSheet {
    /// Character name.
    pub name: String,
    /// Attribute scores (e.g., Agility: 3, Brawn: 4).
    pub attributes: HashMap<String, u32>,
    /// Skill scores (e.g., Melee: 2, Stealth: 1).
    pub skills: HashMap<String, u32>,
    /// Focuses that grant bonuses (e.g., "Blade", "Heavy Armor").
    pub focuses: Vec<String>,
    /// Resource tracks (e.g., Momentum, Stress, Ruin).
    pub tracks: HashMap<String, Track>,
    /// Narrative traits with no mechanical value.
    pub traits: Vec<String>,
}

impl CharacterSheet {
    /// Build a character sheet from an entity's `mechanics.*` properties.
    ///
    /// The ruleset determines which property keys map to attributes vs skills
    /// vs tracks. Properties that don't match any known key are ignored.
    pub fn from_entity(entity: &Entity, ruleset: &RuleSet) -> MechResult<Self> {
        let mut attributes = HashMap::new();
        let mut skills = HashMap::new();
        let mut focuses = Vec::new();
        let mut traits = Vec::new();
        let mut track_overrides: HashMap<String, i32> = HashMap::new();

        // Collect all mechanics.* properties
        for (key, value) in &entity.properties {
            let Some(field) = key.strip_prefix("mechanics.") else {
                continue;
            };

            let field_lower = field.to_lowercase();

            // Check if it's an attribute
            if ruleset
                .attributes
                .iter()
                .any(|a| a.to_lowercase() == field_lower)
            {
                if let Some(v) = extract_u32(value) {
                    let canonical = ruleset
                        .attributes
                        .iter()
                        .find(|a| a.to_lowercase() == field_lower)
                        .cloned()
                        .unwrap_or_else(|| field.to_string());
                    attributes.insert(canonical, v);
                }
                continue;
            }

            // Check if it's a skill
            if ruleset
                .skills
                .iter()
                .any(|s| s.to_lowercase() == field_lower)
            {
                if let Some(v) = extract_u32(value) {
                    let canonical = ruleset
                        .skills
                        .iter()
                        .find(|s| s.to_lowercase() == field_lower)
                        .cloned()
                        .unwrap_or_else(|| field.to_string());
                    skills.insert(canonical, v);
                }
                continue;
            }

            // Check if it's a track override
            if ruleset
                .track_definitions
                .iter()
                .any(|t| t.name.to_lowercase() == field_lower)
            {
                if let Some(v) = extract_i32(value) {
                    let canonical = ruleset
                        .track_definitions
                        .iter()
                        .find(|t| t.name.to_lowercase() == field_lower)
                        .map(|t| t.name.clone())
                        .unwrap_or_else(|| field.to_string());
                    track_overrides.insert(canonical, v);
                }
                continue;
            }

            // Check for focuses and traits
            match field {
                "focus" => {
                    if let MetadataValue::String(s) = value {
                        focuses.push(s.clone());
                    }
                }
                "focuses" => {
                    if let MetadataValue::List(items) = value {
                        for item in items {
                            if let MetadataValue::String(s) = item {
                                focuses.push(s.clone());
                            }
                        }
                    }
                }
                "trait" => {
                    if let MetadataValue::String(s) = value {
                        traits.push(s.clone());
                    }
                }
                "traits" => {
                    if let MetadataValue::List(items) = value {
                        for item in items {
                            if let MetadataValue::String(s) = item {
                                traits.push(s.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Build tracks from ruleset definitions, applying overrides
        let tracks = build_tracks(&ruleset.track_definitions, &track_overrides);

        Ok(Self {
            name: entity.name.clone(),
            attributes,
            skills,
            focuses,
            tracks,
            traits,
        })
    }

    /// Get an attribute value, returning an error if not found.
    pub fn attribute(&self, name: &str) -> MechResult<u32> {
        self.attributes
            .get(name)
            .copied()
            .ok_or_else(|| MechError::UnknownAttribute(name.to_string()))
    }

    /// Get a skill value, returning 0 if not found (untrained).
    pub fn skill(&self, name: &str) -> u32 {
        self.skills.get(name).copied().unwrap_or(0)
    }

    /// Get a mutable reference to a track.
    pub fn track_mut(&mut self, name: &str) -> MechResult<&mut Track> {
        self.tracks
            .get_mut(name)
            .ok_or_else(|| MechError::TrackNotFound(name.to_string()))
    }

    /// Get a reference to a track.
    pub fn track(&self, name: &str) -> MechResult<&Track> {
        self.tracks
            .get(name)
            .ok_or_else(|| MechError::TrackNotFound(name.to_string()))
    }

    /// Returns true if the character has a specific focus.
    pub fn has_focus(&self, focus: &str) -> bool {
        let lower = focus.to_lowercase();
        self.focuses.iter().any(|f| f.to_lowercase() == lower)
    }
}

/// Build tracks from definitions, applying any current-value overrides.
fn build_tracks(
    definitions: &[TrackDefinition],
    overrides: &HashMap<String, i32>,
) -> HashMap<String, Track> {
    definitions
        .iter()
        .map(|def| {
            let current = overrides.get(&def.name).copied().unwrap_or(def.default_max);
            let track = Track::with_range(&def.name, current, def.min, def.default_max);
            (def.name.clone(), track)
        })
        .collect()
}

/// Extract a u32 from a MetadataValue.
fn extract_u32(value: &MetadataValue) -> Option<u32> {
    match value {
        MetadataValue::Integer(n) if *n >= 0 => Some(*n as u32),
        MetadataValue::Float(f) if *f >= 0.0 => Some(*f as u32),
        _ => None,
    }
}

/// Extract an i32 from a MetadataValue.
fn extract_i32(value: &MetadataValue) -> Option<i32> {
    match value {
        MetadataValue::Integer(n) => Some(*n as i32),
        MetadataValue::Float(f) => Some(*f as i32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::entity::{Entity, EntityKind};

    use crate::rules::TrackDefinition;

    fn test_ruleset() -> RuleSet {
        RuleSet {
            name: "test".to_string(),
            check_die: crate::dice::Die::D20,
            default_pool_size: 2,
            resolution: crate::resolution::ResolutionStrategy::Count(
                crate::resolution::CountSuccesses::default(),
            ),
            attributes: vec!["Agility".to_string(), "Brawn".to_string()],
            skills: vec!["Melee".to_string(), "Stealth".to_string()],
            track_definitions: vec![
                TrackDefinition {
                    name: "Stress".to_string(),
                    default_max: 5,
                    min: 0,
                },
                TrackDefinition {
                    name: "Momentum".to_string(),
                    default_max: 6,
                    min: 0,
                },
            ],
            flags: std::collections::HashSet::new(),
        }
    }

    #[test]
    fn from_entity_reads_attributes_and_skills() {
        let ruleset = test_ruleset();
        let mut entity = Entity::new(EntityKind::Character, "Kael");
        entity
            .properties
            .insert("mechanics.agility".to_string(), MetadataValue::Integer(3));
        entity
            .properties
            .insert("mechanics.brawn".to_string(), MetadataValue::Integer(4));
        entity
            .properties
            .insert("mechanics.melee".to_string(), MetadataValue::Integer(2));

        let sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        assert_eq!(sheet.attribute("Agility").unwrap(), 3);
        assert_eq!(sheet.attribute("Brawn").unwrap(), 4);
        assert_eq!(sheet.skill("Melee"), 2);
        assert_eq!(sheet.skill("Stealth"), 0); // untrained
    }

    #[test]
    fn from_entity_reads_focuses() {
        let ruleset = test_ruleset();
        let mut entity = Entity::new(EntityKind::Character, "Kael");
        entity.properties.insert(
            "mechanics.focuses".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("Blade".to_string()),
                MetadataValue::String("Heavy Armor".to_string()),
            ]),
        );

        let sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        assert!(sheet.has_focus("Blade"));
        assert!(sheet.has_focus("heavy armor")); // case insensitive
        assert!(!sheet.has_focus("Shield"));
    }

    #[test]
    fn from_entity_builds_tracks_with_defaults() {
        let ruleset = test_ruleset();
        let entity = Entity::new(EntityKind::Character, "Kael");
        let sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();

        let stress = sheet.track("Stress").unwrap();
        assert_eq!(stress.current, 5);
        assert_eq!(stress.max, 5);

        let momentum = sheet.track("Momentum").unwrap();
        assert_eq!(momentum.current, 6);
        assert_eq!(momentum.max, 6);
    }

    #[test]
    fn from_entity_track_override() {
        let ruleset = test_ruleset();
        let mut entity = Entity::new(EntityKind::Character, "Kael");
        entity
            .properties
            .insert("mechanics.stress".to_string(), MetadataValue::Integer(2));

        let sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        let stress = sheet.track("Stress").unwrap();
        assert_eq!(stress.current, 2);
        assert_eq!(stress.max, 5);
    }

    #[test]
    fn attribute_error() {
        let ruleset = test_ruleset();
        let entity = Entity::new(EntityKind::Character, "Kael");
        let sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        assert!(sheet.attribute("Nonexistent").is_err());
    }

    #[test]
    fn track_error() {
        let ruleset = test_ruleset();
        let entity = Entity::new(EntityKind::Character, "Kael");
        let sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        assert!(sheet.track("Nonexistent").is_err());
    }

    #[test]
    fn track_mut_adjust() {
        let ruleset = test_ruleset();
        let entity = Entity::new(EntityKind::Character, "Kael");
        let mut sheet = CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        let stress = sheet.track_mut("Stress").unwrap();
        stress.adjust(-2);
        assert_eq!(sheet.track("Stress").unwrap().current, 3);
    }
}
