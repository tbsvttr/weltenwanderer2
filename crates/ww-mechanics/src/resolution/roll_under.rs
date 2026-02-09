//! Roll-under resolution (Mothership-style).
//!
//! Roll a single die and check if the result is at or below a target number
//! (usually an attribute value). Doubles (11, 22, 33, etc.) are special:
//! doubles at or below the target are critical successes, doubles above
//! the target are critical failures.

use serde::{Deserialize, Serialize};

use crate::dice::RollResult;
use crate::resolution::Outcome;

/// Configuration for roll-under resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollUnder {
    /// The target number to roll at or under (set per check from attribute).
    pub target_number: u32,
}

impl Default for RollUnder {
    fn default() -> Self {
        Self { target_number: 50 }
    }
}

impl RollUnder {
    /// Resolve a roll by comparing the first die against the target number.
    ///
    /// Success if roll ≤ target. Doubles below = critical success,
    /// doubles above = critical failure.
    pub fn resolve(&self, roll: &RollResult) -> Outcome {
        let value = roll.dice.first().map(|d| d.value).unwrap_or(0);
        if value == 0 {
            return Outcome::Failure;
        }

        let doubles = is_doubles(value);

        if value <= self.target_number {
            let margin = self.target_number - value;
            if doubles {
                Outcome::CriticalSuccess { margin }
            } else {
                Outcome::Success { margin }
            }
        } else if doubles {
            Outcome::CriticalFailure
        } else {
            Outcome::Failure
        }
    }
}

/// Check if a d100 roll shows doubles (11, 22, 33, ..., 99, or 100 as "00").
fn is_doubles(value: u32) -> bool {
    if value == 100 {
        return true; // 00 on percentile dice
    }
    if value > 99 {
        return false;
    }
    let tens = value / 10;
    let ones = value % 10;
    tens == ones
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{DiceTag, Die, DieResult};

    fn make_d100_roll(value: u32) -> RollResult {
        RollResult {
            dice: vec![DieResult {
                die: Die::D100,
                tag: DiceTag::Default,
                value,
            }],
        }
    }

    #[test]
    fn success_below_target() {
        let strategy = RollUnder { target_number: 60 };
        let roll = make_d100_roll(45);
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 15 });
    }

    #[test]
    fn success_at_target() {
        let strategy = RollUnder { target_number: 60 };
        let roll = make_d100_roll(60);
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 0 });
    }

    #[test]
    fn failure_above_target() {
        let strategy = RollUnder { target_number: 60 };
        let roll = make_d100_roll(75);
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn critical_success_doubles_under() {
        let strategy = RollUnder { target_number: 60 };
        let roll = make_d100_roll(33);
        assert_eq!(
            strategy.resolve(&roll),
            Outcome::CriticalSuccess { margin: 27 }
        );
    }

    #[test]
    fn critical_failure_doubles_over() {
        let strategy = RollUnder { target_number: 60 };
        let roll = make_d100_roll(88);
        assert_eq!(strategy.resolve(&roll), Outcome::CriticalFailure);
    }

    #[test]
    fn critical_failure_99() {
        let strategy = RollUnder { target_number: 99 };
        let roll = make_d100_roll(99);
        // 99 is doubles AND ≤ 99, so it's a crit success
        assert_eq!(
            strategy.resolve(&roll),
            Outcome::CriticalSuccess { margin: 0 }
        );
    }

    #[test]
    fn roll_of_100_as_doubles() {
        let strategy = RollUnder { target_number: 50 };
        let roll = make_d100_roll(100);
        // 100 (00 on percentile) is doubles and above 50 → crit failure
        assert_eq!(strategy.resolve(&roll), Outcome::CriticalFailure);
    }

    #[test]
    fn empty_roll() {
        let strategy = RollUnder::default();
        let roll = RollResult::default();
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn doubles_check() {
        assert!(is_doubles(11));
        assert!(is_doubles(22));
        assert!(is_doubles(55));
        assert!(is_doubles(99));
        assert!(is_doubles(100));
        assert!(!is_doubles(12));
        assert!(!is_doubles(50));
        assert!(!is_doubles(1));
    }
}
