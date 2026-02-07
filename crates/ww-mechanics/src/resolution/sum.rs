//! Sum-pool resolution (Blood & Honor-style).
//!
//! Roll a pool of dice and sum all values. If the total meets or exceeds
//! the target number, the check succeeds. Wagered dice add to the pool
//! but are risked — if the check fails, the wagered amount is lost.

use serde::{Deserialize, Serialize};

use crate::dice::RollResult;
use crate::resolution::Outcome;

/// Configuration for sum-pool resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SumPool {
    /// The target number to meet or exceed (default: 10).
    pub target_number: u32,
    /// Extra dice added from wagers (added to the pool before rolling).
    pub wager_bonus: u32,
}

impl Default for SumPool {
    fn default() -> Self {
        Self {
            target_number: 10,
            wager_bonus: 0,
        }
    }
}

impl SumPool {
    /// Resolve a roll by summing all dice and comparing to the target.
    pub fn resolve(&self, roll: &RollResult) -> Outcome {
        let total = roll.total();

        if total == 0 {
            return Outcome::CriticalFailure;
        }

        if total >= self.target_number {
            let margin = total - self.target_number;
            // Beating the target by double or more is a critical success
            if total >= self.target_number * 2 {
                Outcome::CriticalSuccess { margin }
            } else {
                Outcome::Success { margin }
            }
        } else {
            let deficit = self.target_number - total;
            // Missing by only 1-2 is a partial
            if deficit <= 2 {
                Outcome::Partial
            } else {
                Outcome::Failure
            }
        }
    }

    /// Returns how many wagered dice are in this configuration.
    pub fn wager_count(&self) -> u32 {
        self.wager_bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{DiceTag, Die, DieResult};

    fn make_d6_roll(values: &[u32]) -> RollResult {
        RollResult {
            dice: values
                .iter()
                .map(|&v| DieResult {
                    die: Die::D6,
                    tag: DiceTag::Default,
                    value: v,
                })
                .collect(),
        }
    }

    #[test]
    fn success_at_target() {
        let strategy = SumPool::default(); // TN 10
        let roll = make_d6_roll(&[4, 3, 3]); // total 10
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 0 });
    }

    #[test]
    fn success_above_target() {
        let strategy = SumPool::default();
        let roll = make_d6_roll(&[5, 4, 3]); // total 12
        assert_eq!(strategy.resolve(&roll), Outcome::Success { margin: 2 });
    }

    #[test]
    fn critical_success_double_target() {
        let strategy = SumPool::default(); // TN 10
        let roll = make_d6_roll(&[6, 6, 5, 4]); // total 21
        assert_eq!(
            strategy.resolve(&roll),
            Outcome::CriticalSuccess { margin: 11 }
        );
    }

    #[test]
    fn partial_miss_by_one() {
        let strategy = SumPool::default();
        let roll = make_d6_roll(&[4, 3, 2]); // total 9
        assert_eq!(strategy.resolve(&roll), Outcome::Partial);
    }

    #[test]
    fn partial_miss_by_two() {
        let strategy = SumPool::default();
        let roll = make_d6_roll(&[4, 2, 2]); // total 8
        assert_eq!(strategy.resolve(&roll), Outcome::Partial);
    }

    #[test]
    fn failure_miss_by_three() {
        let strategy = SumPool::default();
        let roll = make_d6_roll(&[3, 2, 2]); // total 7
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn failure_low_roll() {
        let strategy = SumPool::default();
        let roll = make_d6_roll(&[1, 1, 1]); // total 3
        assert_eq!(strategy.resolve(&roll), Outcome::Failure);
    }

    #[test]
    fn critical_failure_empty() {
        let strategy = SumPool::default();
        let roll = RollResult::default();
        assert_eq!(strategy.resolve(&roll), Outcome::CriticalFailure);
    }

    #[test]
    fn wager_count() {
        let strategy = SumPool {
            wager_bonus: 3,
            ..SumPool::default()
        };
        assert_eq!(strategy.wager_count(), 3);
    }
}
