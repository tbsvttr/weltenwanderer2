//! Highest-die resolution (Trophy Gold-style).
//!
//! Roll a pool of d6s. The single highest die determines the outcome:
//! - Below `partial_min`: failure
//! - `partial_min` to `success_min - 1`: partial success
//! - `success_min` or above: full success
//!
//! If `dark_die_penalty` is true and the highest die is a dark die,
//! additional consequences apply (e.g., Ruin increases).

use serde::{Deserialize, Serialize};

use crate::dice::{DiceTag, RollResult};
use crate::resolution::Outcome;

/// Configuration for highest-die resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighestDie {
    /// Minimum value for a partial success (default: 4).
    pub partial_min: u32,
    /// Minimum value for a full success (default: 6).
    pub success_min: u32,
    /// Whether the highest being a dark die triggers penalties.
    pub dark_die_penalty: bool,
}

impl Default for HighestDie {
    fn default() -> Self {
        Self {
            partial_min: 4,
            success_min: 6,
            dark_die_penalty: true,
        }
    }
}

impl HighestDie {
    /// Resolve a roll by checking the highest die value.
    pub fn resolve(&self, roll: &RollResult) -> Outcome {
        let highest_value = roll.highest();

        if highest_value == 0 {
            return Outcome::Failure;
        }

        // Check if the highest value comes from a dark die
        let highest_is_dark = roll
            .dice
            .iter()
            .filter(|d| d.value == highest_value)
            .any(|d| d.tag == DiceTag::Dark);

        let base_outcome = if highest_value >= self.success_min {
            let margin = highest_value - self.success_min;
            Outcome::Success { margin }
        } else if highest_value >= self.partial_min {
            Outcome::Partial
        } else {
            Outcome::Failure
        };

        // Dark die penalty downgrades the outcome
        if self.dark_die_penalty && highest_is_dark {
            match base_outcome {
                Outcome::Success { .. } => Outcome::Partial,
                Outcome::Partial => Outcome::Failure,
                other => other,
            }
        } else {
            base_outcome
        }
    }

    /// Returns true if the highest-value die in the roll is a dark die.
    pub fn is_dark_highest(&self, roll: &RollResult) -> bool {
        let highest_value = roll.highest();
        roll.dice
            .iter()
            .filter(|d| d.value == highest_value)
            .any(|d| d.tag == DiceTag::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{Die, DieResult};

    fn make_trophy_roll(light: &[u32], dark: &[u32]) -> RollResult {
        let mut dice: Vec<DieResult> = light
            .iter()
            .map(|&v| DieResult {
                die: Die::D6,
                tag: DiceTag::Light,
                value: v,
            })
            .collect();
        dice.extend(dark.iter().map(|&v| DieResult {
            die: Die::D6,
            tag: DiceTag::Dark,
            value: v,
        }));
        RollResult { dice }
    }

    #[test]
    fn full_success_light() {
        let strategy = HighestDie::default();
        let roll = make_trophy_roll(&[6, 2], &[3]);
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 0 });
    }

    #[test]
    fn partial_success() {
        let strategy = HighestDie::default();
        let roll = make_trophy_roll(&[4, 2], &[1]);
        assert_eq!(strategy.resolve(&roll), Outcome::Partial);
    }

    #[test]
    fn failure() {
        let strategy = HighestDie::default();
        let roll = make_trophy_roll(&[2, 1], &[3]);
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn dark_die_penalty_downgrades_success() {
        let strategy = HighestDie::default();
        // Highest is 6 from dark die → downgraded from success to partial
        let roll = make_trophy_roll(&[3], &[6]);
        assert_eq!(strategy.resolve(&roll), Outcome::Partial);
        assert!(strategy.is_dark_highest(&roll));
    }

    #[test]
    fn dark_die_penalty_downgrades_partial() {
        let strategy = HighestDie::default();
        // Highest is 4 from dark die → downgraded from partial to failure
        let roll = make_trophy_roll(&[1], &[4]);
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn no_dark_penalty_when_disabled() {
        let strategy = HighestDie {
            dark_die_penalty: false,
            ..HighestDie::default()
        };
        let roll = make_trophy_roll(&[3], &[6]);
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 0 });
    }

    #[test]
    fn empty_roll() {
        let strategy = HighestDie::default();
        let roll = RollResult::default();
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn is_dark_highest_check() {
        let strategy = HighestDie::default();
        let roll = make_trophy_roll(&[3], &[6]);
        assert!(strategy.is_dark_highest(&roll));

        let roll2 = make_trophy_roll(&[6], &[3]);
        assert!(!strategy.is_dark_highest(&roll2));
    }
}
