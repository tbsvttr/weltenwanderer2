//! Resolution strategies for interpreting dice rolls.
//!
//! Each TTRPG system resolves dice differently:
//! - **Count successes** (2d20): count dice at or below a target number
//! - **Highest die** (Trophy Gold): check the single highest die value
//! - **Sum pool** (Blood & Honor): sum all dice and compare to a target
//! - **Roll under** (Mothership): roll one die at or below a target value

pub mod count;
pub mod highest;
pub mod roll_under;
pub mod sum;

pub use count::CountSuccesses;
pub use highest::HighestDie;
pub use roll_under::RollUnder;
pub use sum::SumPool;

use serde::{Deserialize, Serialize};

use crate::dice::RollResult;

/// How a dice roll is interpreted to determine success or failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Count dice at or below a target number (2d20-style).
    Count(CountSuccesses),
    /// Check the highest die value against thresholds (Trophy Gold-style).
    Highest(HighestDie),
    /// Sum all dice and compare to a target number (Blood & Honor-style).
    Sum(SumPool),
    /// Roll a single die and check if it's at or below a target (Mothership-style).
    RollUnder(RollUnder),
}

/// The outcome of resolving a dice roll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// An exceptional success beyond normal limits.
    CriticalSuccess {
        /// How far above the threshold the result was.
        margin: u32,
    },
    /// A standard success.
    Success {
        /// How far above the threshold the result was.
        margin: u32,
    },
    /// A mixed result — succeed at a cost.
    Partial,
    /// A standard failure.
    Failure,
    /// A catastrophic failure with additional consequences.
    CriticalFailure,
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CriticalSuccess { margin } => write!(f, "Critical Success (margin {margin})"),
            Self::Success { margin } => write!(f, "Success (margin {margin})"),
            Self::Partial => write!(f, "Partial Success"),
            Self::Failure => write!(f, "Failure"),
            Self::CriticalFailure => write!(f, "Critical Failure"),
        }
    }
}

/// Resolve a dice roll using the given strategy.
pub fn resolve(strategy: &ResolutionStrategy, roll: &RollResult) -> Outcome {
    match strategy {
        ResolutionStrategy::Count(s) => s.resolve(roll),
        ResolutionStrategy::Highest(s) => s.resolve(roll),
        ResolutionStrategy::Sum(s) => s.resolve(roll),
        ResolutionStrategy::RollUnder(s) => s.resolve(roll),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_display() {
        assert_eq!(
            Outcome::CriticalSuccess { margin: 3 }.to_string(),
            "Critical Success (margin 3)"
        );
        assert_eq!(
            Outcome::Success { margin: 1 }.to_string(),
            "Success (margin 1)"
        );
        assert_eq!(Outcome::Partial.to_string(), "Partial Success");
        assert_eq!(Outcome::Failure.to_string(), "Failure");
        assert_eq!(Outcome::CriticalFailure.to_string(), "Critical Failure");
    }
}
