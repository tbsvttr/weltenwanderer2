//! Pre-configured rulesets for common TTRPG systems.
//!
//! These produce the same [`RuleSet`] that the equivalent DSL configuration
//! would, but without requiring a `.ww` file.

use std::collections::HashSet;

use crate::dice::Die;
use crate::resolution::{CountSuccesses, HighestDie, ResolutionStrategy, SumPool};
use crate::rules::{RuleSet, TrackDefinition};

/// 2d20 system (Modiphius-style).
///
/// Roll 2d20 and count successes at or below the target number.
/// A roll at or below the critical threshold scores 2 successes.
/// Features a momentum economy for banking extra successes.
pub fn two_d20() -> RuleSet {
    RuleSet {
        name: "2d20".to_string(),
        check_die: Die::D20,
        default_pool_size: 2,
        resolution: ResolutionStrategy::Count(CountSuccesses {
            target_number: 10,
            critical_threshold: 1,
            successes_needed: 1,
        }),
        attributes: vec![
            "Agility".to_string(),
            "Brawn".to_string(),
            "Coordination".to_string(),
            "Insight".to_string(),
            "Reason".to_string(),
            "Will".to_string(),
        ],
        skills: vec![
            "Athletics".to_string(),
            "Acrobatics".to_string(),
            "Melee".to_string(),
            "Ranged".to_string(),
            "Stealth".to_string(),
            "Observation".to_string(),
            "Survival".to_string(),
            "Persuade".to_string(),
            "Command".to_string(),
            "Society".to_string(),
        ],
        track_definitions: vec![
            TrackDefinition {
                name: "Momentum".to_string(),
                default_max: 6,
                min: 0,
            },
            TrackDefinition {
                name: "Stress".to_string(),
                default_max: 5,
                min: 0,
            },
            TrackDefinition {
                name: "Wounds".to_string(),
                default_max: 5,
                min: 0,
            },
        ],
        flags: HashSet::from(["momentum_economy".to_string()]),
    }
}

/// Trophy Gold system.
///
/// Roll d6 pools with light (player) and dark (risk) dice.
/// The highest single die determines the outcome:
/// 1-3 = failure, 4-5 = partial, 6 = success.
/// If the highest die is dark, Ruin increases.
pub fn trophy_gold() -> RuleSet {
    RuleSet {
        name: "trophy_gold".to_string(),
        check_die: Die::D6,
        default_pool_size: 1,
        resolution: ResolutionStrategy::Highest(HighestDie {
            partial_min: 4,
            success_min: 6,
            dark_die_penalty: true,
        }),
        attributes: vec![
            "Brawn".to_string(),
            "Finesse".to_string(),
            "Mind".to_string(),
            "Will".to_string(),
        ],
        skills: vec![
            "Delving".to_string(),
            "Fighting".to_string(),
            "Rituals".to_string(),
            "Crafting".to_string(),
        ],
        track_definitions: vec![
            TrackDefinition {
                name: "Ruin".to_string(),
                default_max: 6,
                min: 1,
            },
            TrackDefinition {
                name: "Gold".to_string(),
                default_max: 100,
                min: 0,
            },
            TrackDefinition {
                name: "Burdens".to_string(),
                default_max: 6,
                min: 0,
            },
        ],
        flags: HashSet::from(["dark_die_ruin".to_string()]),
    }
}

/// Blood & Honor system.
///
/// Roll a pool of d6s, sum all values, and try to beat target number 10.
/// Players can wager dice for higher stakes. Features Glory and Honor tracks.
pub fn blood_and_honor() -> RuleSet {
    RuleSet {
        name: "blood_and_honor".to_string(),
        check_die: Die::D6,
        default_pool_size: 3,
        resolution: ResolutionStrategy::Sum(SumPool {
            target_number: 10,
            wager_bonus: 0,
        }),
        attributes: vec![
            "Beauty".to_string(),
            "Courage".to_string(),
            "Cunning".to_string(),
            "Prowess".to_string(),
            "Strength".to_string(),
            "Wisdom".to_string(),
        ],
        skills: vec![
            "Art".to_string(),
            "Etiquette".to_string(),
            "Hunting".to_string(),
            "Warfare".to_string(),
            "Athletics".to_string(),
            "Intimidate".to_string(),
        ],
        track_definitions: vec![
            TrackDefinition {
                name: "Glory".to_string(),
                default_max: 10,
                min: 0,
            },
            TrackDefinition {
                name: "Honor".to_string(),
                default_max: 10,
                min: 0,
            },
            TrackDefinition {
                name: "Wounds".to_string(),
                default_max: 5,
                min: 0,
            },
        ],
        flags: HashSet::from(["wager_system".to_string()]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_d20_preset() {
        let rs = two_d20();
        assert_eq!(rs.name, "2d20");
        assert_eq!(rs.check_die, Die::D20);
        assert_eq!(rs.default_pool_size, 2);
        assert_eq!(rs.attributes.len(), 6);
        assert!(rs.has_flag("momentum_economy"));
        assert!(matches!(rs.resolution, ResolutionStrategy::Count(_)));
    }

    #[test]
    fn trophy_gold_preset() {
        let rs = trophy_gold();
        assert_eq!(rs.name, "trophy_gold");
        assert_eq!(rs.check_die, Die::D6);
        assert_eq!(rs.default_pool_size, 1);
        assert!(rs.has_flag("dark_die_ruin"));
        assert!(matches!(rs.resolution, ResolutionStrategy::Highest(_)));
        // Ruin starts at min 1
        let ruin = rs
            .track_definitions
            .iter()
            .find(|t| t.name == "Ruin")
            .unwrap();
        assert_eq!(ruin.min, 1);
    }

    #[test]
    fn blood_and_honor_preset() {
        let rs = blood_and_honor();
        assert_eq!(rs.name, "blood_and_honor");
        assert_eq!(rs.check_die, Die::D6);
        assert_eq!(rs.default_pool_size, 3);
        assert!(rs.has_flag("wager_system"));
        assert!(matches!(rs.resolution, ResolutionStrategy::Sum(_)));
        assert_eq!(rs.attributes.len(), 6);
    }

    #[test]
    fn presets_have_tracks() {
        for rs in [two_d20(), trophy_gold(), blood_and_honor()] {
            assert!(
                !rs.track_definitions.is_empty(),
                "{} has no track definitions",
                rs.name
            );
        }
    }
}
