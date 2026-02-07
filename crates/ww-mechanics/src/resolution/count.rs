//! Count-successes resolution (2d20-style).
//!
//! Roll multiple d20s. Each die at or below the target number scores one success.
//! A die at or below the critical threshold scores two successes instead.
//! Meeting or exceeding `successes_needed` is a success.

use serde::{Deserialize, Serialize};

use crate::dice::RollResult;
use crate::resolution::Outcome;

/// Configuration for count-successes resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountSuccesses {
    /// Roll at or below this value to score a success.
    pub target_number: u32,
    /// Roll at or below this value to score two successes (critical range).
    pub critical_threshold: u32,
    /// How many successes are needed for a standard success.
    pub successes_needed: u32,
}

impl Default for CountSuccesses {
    fn default() -> Self {
        Self {
            target_number: 10,
            critical_threshold: 1,
            successes_needed: 1,
        }
    }
}

impl CountSuccesses {
    /// Resolve a roll by counting successes.
    pub fn resolve(&self, roll: &RollResult) -> Outcome {
        let mut successes: u32 = 0;
        let mut has_natural_20 = false;

        for die in &roll.dice {
            if die.value <= self.critical_threshold {
                successes += 2;
            } else if die.value <= self.target_number {
                successes += 1;
            }
            if die.value == die.die.sides() {
                has_natural_20 = true;
            }
        }

        if successes == 0 && has_natural_20 {
            return Outcome::CriticalFailure;
        }

        if successes >= self.successes_needed {
            let margin = successes - self.successes_needed;
            if successes >= self.successes_needed * 2 {
                Outcome::CriticalSuccess { margin }
            } else {
                Outcome::Success { margin }
            }
        } else if successes > 0 {
            Outcome::Partial
        } else {
            Outcome::Failure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{DiceTag, Die, DieResult};

    fn make_d20_roll(values: &[u32]) -> RollResult {
        RollResult {
            dice: values
                .iter()
                .map(|&v| DieResult {
                    die: Die::D20,
                    tag: DiceTag::Default,
                    value: v,
                })
                .collect(),
        }
    }

    #[test]
    fn two_d20_both_succeed() {
        let strategy = CountSuccesses {
            target_number: 12,
            critical_threshold: 2,
            successes_needed: 1,
        };
        let roll = make_d20_roll(&[5, 8]);
        assert_eq!(
            strategy.resolve(&roll),
            Outcome::CriticalSuccess { margin: 1 }
        );
    }

    #[test]
    fn two_d20_one_succeeds() {
        let strategy = CountSuccesses {
            target_number: 10,
            critical_threshold: 1,
            successes_needed: 1,
        };
        let roll = make_d20_roll(&[7, 15]);
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 0 });
    }

    #[test]
    fn two_d20_none_succeed() {
        let strategy = CountSuccesses {
            target_number: 10,
            critical_threshold: 1,
            successes_needed: 1,
        };
        let roll = make_d20_roll(&[15, 18]);
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn critical_hit_doubles() {
        let strategy = CountSuccesses {
            target_number: 10,
            critical_threshold: 2,
            successes_needed: 2,
        };
        // A roll of 1 gives 2 successes (critical)
        let roll = make_d20_roll(&[1, 15]);
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 0 });
    }

    #[test]
    fn natural_max_with_no_successes_is_crit_fail() {
        let strategy = CountSuccesses {
            target_number: 5,
            critical_threshold: 1,
            successes_needed: 1,
        };
        let roll = make_d20_roll(&[20, 18]);
        assert_eq!(strategy.resolve(&roll), Outcome::CriticalFailure);
    }

    #[test]
    fn partial_success() {
        let strategy = CountSuccesses {
            target_number: 10,
            critical_threshold: 1,
            successes_needed: 3,
        };
        let roll = make_d20_roll(&[5, 8, 15, 18]);
        // 2 successes out of 3 needed
        assert_eq!(strategy.resolve(&roll), Outcome::Partial);
    }

    #[test]
    fn default_values() {
        let strategy = CountSuccesses::default();
        assert_eq!(strategy.target_number, 10);
        assert_eq!(strategy.critical_threshold, 1);
        assert_eq!(strategy.successes_needed, 1);
    }
}
