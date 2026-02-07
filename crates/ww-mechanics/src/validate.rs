//! Validation of mechanics configuration across a compiled world.
//!
//! Checks that the ruleset is well-formed and that character mechanics
//! blocks reference only attributes, skills, and tracks defined in the
//! ruleset.

use ww_core::entity::{EntityKind, MetadataValue};
use ww_core::world::World;

use crate::rules::RuleSet;

/// A warning or error found during mechanics validation.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// The entity name where the issue was found.
    pub entity: String,
    /// A human-readable description of the issue.
    pub message: String,
    /// Whether this is an error (true) or a warning (false).
    pub is_error: bool,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = if self.is_error { "error" } else { "warning" };
        write!(f, "{level}: {}: {}", self.entity, self.message)
    }
}

/// Validate mechanics configuration in a compiled world.
///
/// Returns a list of issues found. If no ruleset is defined, returns
/// an empty list (mechanics are optional). If a ruleset is present,
/// validates that all character mechanics blocks are consistent with it.
pub fn validate_world(world: &World) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Try to load the ruleset — if none exists, mechanics are optional
    let ruleset = match RuleSet::from_world(world) {
        Ok(rs) => rs,
        Err(crate::error::MechError::NoMechanicsConfig) => return issues,
        Err(e) => {
            // Find the entity that has mechanics.system to report against
            let entity_name = world
                .all_entities()
                .find(|e| e.properties.contains_key("mechanics.system"))
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            issues.push(ValidationIssue {
                entity: entity_name,
                message: format!("invalid ruleset: {e}"),
                is_error: true,
            });
            return issues;
        }
    };

    // Validate the ruleset itself
    validate_ruleset(&ruleset, &mut issues);

    // Validate each character's mechanics block against the ruleset
    for entity in world.entities_by_kind(&EntityKind::Character) {
        validate_character(entity, &ruleset, &mut issues);
    }

    issues
}

/// Validate ruleset internal consistency.
fn validate_ruleset(ruleset: &RuleSet, issues: &mut Vec<ValidationIssue>) {
    if ruleset.attributes.is_empty() {
        issues.push(ValidationIssue {
            entity: format!("ruleset '{}'", ruleset.name),
            message: "no attributes defined".to_string(),
            is_error: false,
        });
    }

    if ruleset.skills.is_empty() {
        issues.push(ValidationIssue {
            entity: format!("ruleset '{}'", ruleset.name),
            message: "no skills defined".to_string(),
            is_error: false,
        });
    }

    for track in &ruleset.track_definitions {
        if track.default_max <= track.min {
            issues.push(ValidationIssue {
                entity: format!("ruleset '{}'", ruleset.name),
                message: format!(
                    "track '{}' has max ({}) <= min ({})",
                    track.name, track.default_max, track.min
                ),
                is_error: true,
            });
        }
    }
}

/// Validate a character entity's mechanics block against the ruleset.
fn validate_character(
    entity: &ww_core::entity::Entity,
    ruleset: &RuleSet,
    issues: &mut Vec<ValidationIssue>,
) {
    let mech_keys: Vec<(&String, &MetadataValue)> = entity
        .properties
        .iter()
        .filter(|(k, _)| k.starts_with("mechanics."))
        .collect();

    // No mechanics block — that's fine, not all characters need stats
    if mech_keys.is_empty() {
        return;
    }

    let attr_lower: Vec<String> = ruleset
        .attributes
        .iter()
        .map(|a| a.to_lowercase())
        .collect();
    let skill_lower: Vec<String> = ruleset.skills.iter().map(|s| s.to_lowercase()).collect();
    let track_lower: Vec<String> = ruleset
        .track_definitions
        .iter()
        .map(|t| t.name.to_lowercase())
        .collect();

    for (key, value) in &mech_keys {
        let field = match key.strip_prefix("mechanics.") {
            Some(f) => f,
            None => continue,
        };

        let field_lower = field.to_lowercase();

        // Skip known meta-fields
        if matches!(
            field,
            "focuses"
                | "focus"
                | "traits"
                | "trait"
                | "system"
                | "check_die"
                | "pool_size"
                | "resolution"
        ) {
            continue;
        }

        let is_attribute = attr_lower.contains(&field_lower);
        let is_skill = skill_lower.contains(&field_lower);
        let is_track = track_lower.contains(&field_lower);

        if !is_attribute && !is_skill && !is_track {
            issues.push(ValidationIssue {
                entity: entity.name.clone(),
                message: format!(
                    "unknown mechanics field '{field}' — not an attribute, skill, or track in ruleset '{}'",
                    ruleset.name
                ),
                is_error: false,
            });
            continue;
        }

        // Validate numeric types for attributes and skills
        if is_attribute || is_skill {
            match value {
                MetadataValue::Integer(n) if *n < 0 => {
                    issues.push(ValidationIssue {
                        entity: entity.name.clone(),
                        message: format!("'{field}' has negative value {n}"),
                        is_error: true,
                    });
                }
                MetadataValue::Integer(_) | MetadataValue::Float(_) => {}
                _ => {
                    issues.push(ValidationIssue {
                        entity: entity.name.clone(),
                        message: format!("'{field}' should be a number, got {value}"),
                        is_error: true,
                    });
                }
            }
        }

        // Validate track values are within bounds
        if is_track
            && let Some(track_def) = ruleset
                .track_definitions
                .iter()
                .find(|t| t.name.to_lowercase() == field_lower)
            && let MetadataValue::Integer(n) = value
        {
            let n = *n as i32;
            if n < track_def.min || n > track_def.default_max {
                issues.push(ValidationIssue {
                    entity: entity.name.clone(),
                    message: format!(
                        "track '{}' value {n} is outside bounds [{}, {}]",
                        track_def.name, track_def.min, track_def.default_max
                    ),
                    is_error: true,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::entity::Entity;
    use ww_core::world::WorldMeta;

    fn make_world_with_ruleset_and_character(
        ruleset_props: Vec<(&str, MetadataValue)>,
        char_props: Vec<(&str, MetadataValue)>,
    ) -> World {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut rs = Entity::new(EntityKind::Custom("ruleset".to_string()), "Rules");
        for (k, v) in ruleset_props {
            rs.properties.insert(k.to_string(), v);
        }
        let _ = world.add_entity(rs);

        if !char_props.is_empty() {
            let mut ch = Entity::new(EntityKind::Character, "Hero");
            for (k, v) in char_props {
                ch.properties.insert(k.to_string(), v);
            }
            let _ = world.add_entity(ch);
        }
        world
    }

    fn basic_ruleset_props() -> Vec<(&'static str, MetadataValue)> {
        vec![
            (
                "mechanics.system",
                MetadataValue::String("2d20".to_string()),
            ),
            (
                "mechanics.check_die",
                MetadataValue::String("d20".to_string()),
            ),
            (
                "mechanics.resolution",
                MetadataValue::String("count_successes".to_string()),
            ),
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
                MetadataValue::List(vec![MetadataValue::String("Stress:5:0".to_string())]),
            ),
        ]
    }

    #[test]
    fn no_mechanics_is_valid() {
        let world = World::new(WorldMeta::new("Empty"));
        let issues = validate_world(&world);
        assert!(issues.is_empty());
    }

    #[test]
    fn valid_character_no_issues() {
        let world = make_world_with_ruleset_and_character(
            basic_ruleset_props(),
            vec![
                ("mechanics.agility", MetadataValue::Integer(10)),
                ("mechanics.brawn", MetadataValue::Integer(8)),
                ("mechanics.melee", MetadataValue::Integer(3)),
                ("mechanics.stress", MetadataValue::Integer(0)),
            ],
        );
        let issues = validate_world(&world);
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn unknown_field_warns() {
        let world = make_world_with_ruleset_and_character(
            basic_ruleset_props(),
            vec![("mechanics.magic", MetadataValue::Integer(5))],
        );
        let issues = validate_world(&world);
        assert_eq!(issues.len(), 1);
        assert!(!issues[0].is_error);
        assert!(
            issues[0]
                .message
                .contains("unknown mechanics field 'magic'")
        );
    }

    #[test]
    fn negative_attribute_errors() {
        let world = make_world_with_ruleset_and_character(
            basic_ruleset_props(),
            vec![("mechanics.agility", MetadataValue::Integer(-3))],
        );
        let issues = validate_world(&world);
        assert!(
            issues
                .iter()
                .any(|i| i.is_error && i.message.contains("negative"))
        );
    }

    #[test]
    fn track_out_of_bounds_errors() {
        let world = make_world_with_ruleset_and_character(
            basic_ruleset_props(),
            vec![("mechanics.stress", MetadataValue::Integer(99))],
        );
        let issues = validate_world(&world);
        assert!(
            issues
                .iter()
                .any(|i| i.is_error && i.message.contains("outside bounds"))
        );
    }

    #[test]
    fn invalid_ruleset_resolution() {
        let world = make_world_with_ruleset_and_character(
            vec![
                (
                    "mechanics.system",
                    MetadataValue::String("unknown_system".to_string()),
                ),
                (
                    "mechanics.resolution",
                    MetadataValue::String("bogus".to_string()),
                ),
            ],
            vec![],
        );
        let issues = validate_world(&world);
        assert!(
            issues
                .iter()
                .any(|i| i.is_error && i.message.contains("invalid ruleset"))
        );
    }

    #[test]
    fn track_max_less_than_min_errors() {
        let mut props = basic_ruleset_props();
        // Replace tracks with an invalid one
        props.retain(|(k, _)| *k != "mechanics.tracks");
        props.push((
            "mechanics.tracks",
            MetadataValue::List(vec![MetadataValue::String("Bad:0:5".to_string())]),
        ));
        let world = make_world_with_ruleset_and_character(props, vec![]);
        let issues = validate_world(&world);
        assert!(
            issues
                .iter()
                .any(|i| i.is_error && i.message.contains("max (0) <= min (5)"))
        );
    }

    #[test]
    fn character_without_mechanics_is_fine() {
        let world = make_world_with_ruleset_and_character(
            basic_ruleset_props(),
            vec![], // No mechanics on the character
        );
        // Add a character with no mechanics
        // (make_world_with_ruleset_and_character skips if char_props is empty)
        // Use add_entity directly
        let mut world2 = world;
        let ch = Entity::new(EntityKind::Character, "Villager");
        let _ = world2.add_entity(ch);
        let issues = validate_world(&world2);
        assert!(issues.is_empty());
    }

    #[test]
    fn non_numeric_attribute_errors() {
        let world = make_world_with_ruleset_and_character(
            basic_ruleset_props(),
            vec![(
                "mechanics.agility",
                MetadataValue::String("fast".to_string()),
            )],
        );
        let issues = validate_world(&world);
        assert!(
            issues
                .iter()
                .any(|i| i.is_error && i.message.contains("should be a number"))
        );
    }
}
