//! Rules engine: rulesets, checks, and results.
//!
//! A [`RuleSet`] defines the complete mechanical framework for a game system.
//! It can be loaded from a world's `mechanics.*` properties via [`RuleSet::from_world`],
//! or constructed programmatically using the preset functions in [`preset`].

pub mod preset;

use std::collections::HashSet;

use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use ww_core::entity::MetadataValue;
use ww_core::world::World;

use crate::dice::{DicePool, DiceTag, Die, RollResult};
use crate::error::{MechError, MechResult};
use crate::resolution::{self, CountSuccesses, HighestDie, Outcome, ResolutionStrategy, SumPool};

/// Definition of a resource track in a ruleset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDefinition {
    /// Track name (e.g., "Momentum", "Ruin", "Honor").
    pub name: String,
    /// Default maximum value.
    pub default_max: i32,
    /// Minimum value (usually 0).
    pub min: i32,
}

/// A complete game system configuration.
#[derive(Debug, Clone)]
pub struct RuleSet {
    /// System name (e.g., "2d20", "trophy_gold", "blood_and_honor").
    pub name: String,
    /// The die type used for standard checks.
    pub check_die: Die,
    /// Default number of dice in the pool.
    pub default_pool_size: u32,
    /// How dice rolls are resolved.
    pub resolution: ResolutionStrategy,
    /// Named attributes in this system (e.g., Agility, Brawn).
    pub attributes: Vec<String>,
    /// Named skills in this system (e.g., Melee, Stealth).
    pub skills: Vec<String>,
    /// Track definitions (e.g., Momentum:6:0, Stress:5:0).
    pub track_definitions: Vec<TrackDefinition>,
    /// System flags (e.g., "momentum_economy", "wager_system").
    pub flags: HashSet<String>,
}

impl RuleSet {
    /// Load a ruleset from the world's `mechanics.*` properties.
    ///
    /// Scans all entities for one with `mechanics.system` set, then reads
    /// the remaining `mechanics.*` properties to build the ruleset.
    pub fn from_world(world: &World) -> MechResult<Self> {
        // Find the entity with mechanics.system defined
        let config_entity = world
            .all_entities()
            .find(|e| e.properties.contains_key("mechanics.system"))
            .ok_or(MechError::NoMechanicsConfig)?;

        let props = &config_entity.properties;

        let name = extract_string(props, "mechanics.system")
            .ok_or_else(|| MechError::InvalidConfig("missing mechanics.system".to_string()))?;

        let check_die_str =
            extract_string(props, "mechanics.check_die").unwrap_or("d20".to_string());
        let check_die = Die::from_str_tag(&check_die_str).ok_or_else(|| {
            MechError::InvalidConfig(format!("invalid check_die: {check_die_str}"))
        })?;

        let default_pool_size = extract_u32(props, "mechanics.pool_size").unwrap_or(2);

        let resolution = build_resolution(props, &name)?;

        let attributes = extract_string_list(props, "mechanics.attributes");
        let skills = extract_string_list(props, "mechanics.skills");
        let track_definitions = parse_track_definitions(props);
        let flags = extract_string_list(props, "mechanics.flags")
            .into_iter()
            .collect();

        Ok(Self {
            name,
            check_die,
            default_pool_size,
            resolution,
            attributes,
            skills,
            track_definitions,
            flags,
        })
    }

    /// Returns true if the given system flag is enabled.
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }
}

/// A request to perform a mechanical check.
#[derive(Debug, Clone, Default)]
pub struct CheckRequest {
    /// Which attribute to test (optional — some systems don't use attributes).
    pub attribute: Option<String>,
    /// Which skill to test (optional).
    pub skill: Option<String>,
    /// Modifier applied to the target number or pool size.
    pub modifier: i32,
    /// Extra dice added to the pool.
    pub extra_dice: u32,
    /// Override the default difficulty/target number.
    pub difficulty: Option<u32>,
}

/// The result of performing a check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// The raw dice roll.
    pub roll: RollResult,
    /// The resolved outcome.
    pub outcome: Outcome,
    /// Side effects triggered by the check.
    pub effects: Vec<CheckEffect>,
}

/// A side effect produced by a check resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckEffect {
    /// Adjust a resource track by a delta.
    TrackAdjust {
        /// Name of the track to adjust.
        track: String,
        /// Amount to change (positive or negative).
        delta: i32,
    },
    /// Generate or spend momentum (2d20 system).
    Momentum(
        /// Positive = generated, negative = spent.
        i32,
    ),
    /// A complication occurred.
    Complication(
        /// Description of the complication.
        String,
    ),
}

impl std::fmt::Display for CheckEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrackAdjust { track, delta } => {
                if *delta >= 0 {
                    write!(f, "{track} +{delta}")
                } else {
                    write!(f, "{track} {delta}")
                }
            }
            Self::Momentum(n) => {
                if *n >= 0 {
                    write!(f, "Momentum +{n}")
                } else {
                    write!(f, "Momentum {n}")
                }
            }
            Self::Complication(desc) => write!(f, "Complication: {desc}"),
        }
    }
}

/// Perform a mechanical check using a ruleset and character sheet.
pub fn perform_check(
    ruleset: &RuleSet,
    sheet: &crate::sheet::CharacterSheet,
    request: &CheckRequest,
    rng: &mut StdRng,
) -> MechResult<CheckResult> {
    // Build the dice pool
    let pool_size =
        (ruleset.default_pool_size as i32 + request.modifier + request.extra_dice as i32).max(1)
            as u32;

    let pool = DicePool::new().add(ruleset.check_die, pool_size);
    let roll = pool.roll(rng);

    // Adjust resolution strategy based on request
    let strategy = apply_check_modifiers(ruleset, sheet, request)?;
    let outcome = resolution::resolve(&strategy, &roll);

    // Generate effects based on system
    let effects = generate_effects(ruleset, &roll, &outcome);

    Ok(CheckResult {
        roll,
        outcome,
        effects,
    })
}

/// Adjust the resolution strategy based on the character's stats and request.
fn apply_check_modifiers(
    ruleset: &RuleSet,
    sheet: &crate::sheet::CharacterSheet,
    request: &CheckRequest,
) -> MechResult<ResolutionStrategy> {
    let mut strategy = ruleset.resolution.clone();

    match &mut strategy {
        ResolutionStrategy::Count(count) => {
            // In 2d20: attribute sets the TN, skill gives bonus successes
            if let Some(ref attr) = request.attribute {
                count.target_number = sheet.attribute(attr)?;
            }
            if let Some(ref skill) = request.skill {
                let skill_val = sheet.skill(skill);
                // Having the skill doesn't change TN but focuses do
                // Keep it simple: skill > 0 means trained
                let _ = skill_val;
            }
            if let Some(difficulty) = request.difficulty {
                count.successes_needed = difficulty;
            }
        }
        ResolutionStrategy::Sum(sum) => {
            if let Some(difficulty) = request.difficulty {
                sum.target_number = difficulty;
            }
        }
        ResolutionStrategy::Highest(highest) => {
            if let Some(difficulty) = request.difficulty {
                highest.success_min = difficulty;
            }
        }
    }

    Ok(strategy)
}

/// Generate side effects based on the outcome and system flags.
fn generate_effects(ruleset: &RuleSet, roll: &RollResult, outcome: &Outcome) -> Vec<CheckEffect> {
    let mut effects = Vec::new();

    // 2d20 momentum economy
    if ruleset.has_flag("momentum_economy") {
        match outcome {
            Outcome::Success { margin } | Outcome::CriticalSuccess { margin } => {
                if *margin > 0 {
                    effects.push(CheckEffect::Momentum(*margin as i32));
                }
            }
            Outcome::CriticalFailure => {
                effects.push(CheckEffect::Complication(
                    "Critical failure complication".to_string(),
                ));
            }
            _ => {}
        }
    }

    // Trophy Gold ruin tracking
    if ruleset.has_flag("dark_die_ruin") {
        let highest_val = roll.highest();
        let highest_is_dark = roll
            .dice
            .iter()
            .filter(|d| d.value == highest_val)
            .any(|d| d.tag == DiceTag::Dark);
        if highest_is_dark {
            effects.push(CheckEffect::TrackAdjust {
                track: "Ruin".to_string(),
                delta: 1,
            });
        }
    }

    effects
}

// --- Helper functions for parsing world properties ---

/// Extract a string from a property map.
fn extract_string(
    props: &std::collections::HashMap<String, MetadataValue>,
    key: &str,
) -> Option<String> {
    match props.get(key)? {
        MetadataValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// Extract a u32 from a property map.
fn extract_u32(props: &std::collections::HashMap<String, MetadataValue>, key: &str) -> Option<u32> {
    match props.get(key)? {
        MetadataValue::Integer(n) if *n >= 0 => Some(*n as u32),
        MetadataValue::Float(f) if *f >= 0.0 => Some(*f as u32),
        _ => None,
    }
}

/// Extract a boolean from a property map.
fn extract_bool(
    props: &std::collections::HashMap<String, MetadataValue>,
    key: &str,
) -> Option<bool> {
    match props.get(key)? {
        MetadataValue::Boolean(b) => Some(*b),
        MetadataValue::String(s) => match s.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Extract a list of strings from a property map.
fn extract_string_list(
    props: &std::collections::HashMap<String, MetadataValue>,
    key: &str,
) -> Vec<String> {
    match props.get(key) {
        Some(MetadataValue::List(items)) => items
            .iter()
            .filter_map(|v| match v {
                MetadataValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Parse track definitions from "mechanics.tracks" property.
///
/// Expected format: list of strings like `"Name:max:min"`.
fn parse_track_definitions(
    props: &std::collections::HashMap<String, MetadataValue>,
) -> Vec<TrackDefinition> {
    extract_string_list(props, "mechanics.tracks")
        .iter()
        .filter_map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            match parts.as_slice() {
                [name, max, min] => Some(TrackDefinition {
                    name: (*name).to_string(),
                    default_max: max.parse().ok()?,
                    min: min.parse().ok()?,
                }),
                [name, max] => Some(TrackDefinition {
                    name: (*name).to_string(),
                    default_max: max.parse().ok()?,
                    min: 0,
                }),
                _ => None,
            }
        })
        .collect()
}

/// Build a resolution strategy from properties and system name.
fn build_resolution(
    props: &std::collections::HashMap<String, MetadataValue>,
    system_name: &str,
) -> MechResult<ResolutionStrategy> {
    let resolution_type =
        extract_string(props, "mechanics.resolution").unwrap_or_else(|| system_name.to_string());

    match resolution_type.as_str() {
        "count_successes" | "2d20" => {
            let target_number = extract_u32(props, "mechanics.target_number").unwrap_or(10);
            let critical_threshold =
                extract_u32(props, "mechanics.critical_threshold").unwrap_or(1);
            let successes_needed = extract_u32(props, "mechanics.successes_needed").unwrap_or(1);
            Ok(ResolutionStrategy::Count(CountSuccesses {
                target_number,
                critical_threshold,
                successes_needed,
            }))
        }
        "highest_die" | "trophy_gold" => {
            let partial_min = extract_u32(props, "mechanics.partial_min").unwrap_or(4);
            let success_min = extract_u32(props, "mechanics.success_min").unwrap_or(6);
            let dark_die_penalty =
                extract_bool(props, "mechanics.dark_die_penalty").unwrap_or(true);
            Ok(ResolutionStrategy::Highest(HighestDie {
                partial_min,
                success_min,
                dark_die_penalty,
            }))
        }
        "sum_pool" | "blood_and_honor" => {
            let target_number = extract_u32(props, "mechanics.target_number").unwrap_or(10);
            let wager_bonus = extract_u32(props, "mechanics.wager_bonus").unwrap_or(0);
            Ok(ResolutionStrategy::Sum(SumPool {
                target_number,
                wager_bonus,
            }))
        }
        other => Err(MechError::InvalidConfig(format!(
            "unknown resolution type: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use ww_core::entity::{Entity, EntityKind};
    use ww_core::world::WorldMeta;

    fn make_world_with_mechanics(props: Vec<(&str, MetadataValue)>) -> World {
        let mut world = World::new(WorldMeta::new("Test World"));
        let mut entity = Entity::new(EntityKind::Custom("ruleset".to_string()), "Game Rules");
        for (key, value) in props {
            entity.properties.insert(key.to_string(), value);
        }
        let _ = world.add_entity(entity);
        world
    }

    #[test]
    fn from_world_2d20() {
        let world = make_world_with_mechanics(vec![
            (
                "mechanics.system",
                MetadataValue::String("2d20".to_string()),
            ),
            (
                "mechanics.check_die",
                MetadataValue::String("d20".to_string()),
            ),
            ("mechanics.pool_size", MetadataValue::Integer(2)),
            (
                "mechanics.resolution",
                MetadataValue::String("count_successes".to_string()),
            ),
            ("mechanics.target_number", MetadataValue::Integer(10)),
            (
                "mechanics.attributes",
                MetadataValue::List(vec![
                    MetadataValue::String("Agility".to_string()),
                    MetadataValue::String("Brawn".to_string()),
                ]),
            ),
            (
                "mechanics.skills",
                MetadataValue::List(vec![MetadataValue::String("Melee".to_string())]),
            ),
            (
                "mechanics.tracks",
                MetadataValue::List(vec![
                    MetadataValue::String("Momentum:6:0".to_string()),
                    MetadataValue::String("Stress:5:0".to_string()),
                ]),
            ),
            (
                "mechanics.flags",
                MetadataValue::List(vec![MetadataValue::String("momentum_economy".to_string())]),
            ),
        ]);

        let ruleset = RuleSet::from_world(&world).unwrap();
        assert_eq!(ruleset.name, "2d20");
        assert_eq!(ruleset.check_die, Die::D20);
        assert_eq!(ruleset.default_pool_size, 2);
        assert_eq!(ruleset.attributes.len(), 2);
        assert_eq!(ruleset.skills.len(), 1);
        assert_eq!(ruleset.track_definitions.len(), 2);
        assert!(ruleset.has_flag("momentum_economy"));
    }

    #[test]
    fn from_world_trophy_gold() {
        let world = make_world_with_mechanics(vec![
            (
                "mechanics.system",
                MetadataValue::String("trophy_gold".to_string()),
            ),
            (
                "mechanics.check_die",
                MetadataValue::String("d6".to_string()),
            ),
            ("mechanics.pool_size", MetadataValue::Integer(1)),
            (
                "mechanics.resolution",
                MetadataValue::String("highest_die".to_string()),
            ),
            ("mechanics.partial_min", MetadataValue::Integer(4)),
            ("mechanics.success_min", MetadataValue::Integer(6)),
            (
                "mechanics.tracks",
                MetadataValue::List(vec![MetadataValue::String("Ruin:6:1".to_string())]),
            ),
        ]);

        let ruleset = RuleSet::from_world(&world).unwrap();
        assert_eq!(ruleset.name, "trophy_gold");
        assert_eq!(ruleset.check_die, Die::D6);
        assert_eq!(ruleset.track_definitions[0].name, "Ruin");
        assert_eq!(ruleset.track_definitions[0].min, 1);
    }

    #[test]
    fn from_world_no_config() {
        let world = World::new(WorldMeta::new("Empty"));
        assert!(matches!(
            RuleSet::from_world(&world),
            Err(MechError::NoMechanicsConfig)
        ));
    }

    #[test]
    fn perform_check_2d20() {
        let ruleset = preset::two_d20();
        let mut entity = Entity::new(EntityKind::Character, "Test");
        entity
            .properties
            .insert("mechanics.agility".to_string(), MetadataValue::Integer(12));
        entity
            .properties
            .insert("mechanics.melee".to_string(), MetadataValue::Integer(2));

        let sheet = crate::sheet::CharacterSheet::from_entity(&entity, &ruleset).unwrap();
        let request = CheckRequest {
            attribute: Some("Agility".to_string()),
            difficulty: Some(1),
            ..CheckRequest::default()
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = perform_check(&ruleset, &sheet, &request, &mut rng).unwrap();
        // With seed 42 and TN 12, we should get a deterministic result
        assert!(!result.roll.dice.is_empty());
    }

    #[test]
    fn check_effect_display() {
        assert_eq!(
            CheckEffect::TrackAdjust {
                track: "Stress".to_string(),
                delta: -2
            }
            .to_string(),
            "Stress -2"
        );
        assert_eq!(CheckEffect::Momentum(3).to_string(), "Momentum +3");
        assert_eq!(
            CheckEffect::Complication("oops".to_string()).to_string(),
            "Complication: oops"
        );
    }

    #[test]
    fn parse_track_definitions_various_formats() {
        let mut props = std::collections::HashMap::new();
        props.insert(
            "mechanics.tracks".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("HP:10:0".to_string()),
                MetadataValue::String("Ruin:6:1".to_string()),
                MetadataValue::String("Gold:100".to_string()),
            ]),
        );
        let defs = parse_track_definitions(&props);
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].name, "HP");
        assert_eq!(defs[0].default_max, 10);
        assert_eq!(defs[0].min, 0);
        assert_eq!(defs[1].name, "Ruin");
        assert_eq!(defs[1].min, 1);
        assert_eq!(defs[2].name, "Gold");
        assert_eq!(defs[2].default_max, 100);
        assert_eq!(defs[2].min, 0);
    }
}
