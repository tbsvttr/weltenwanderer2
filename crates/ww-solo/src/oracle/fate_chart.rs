//! Fate chart oracle for yes/no questions.
//!
//! The fate chart maps a combination of question likelihood and current chaos
//! factor to a probability. A d100 roll determines the answer: Yes, No,
//! Exceptional Yes, or Exceptional No. Certain rolls also trigger random events.

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use super::event::{RandomEvent, generate_random_event};
use super::tables::OracleConfig;

/// How likely the player thinks the answer is "Yes".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Likelihood {
    /// Almost certainly not (5% base).
    Impossible,
    /// Extremely unlikely (10% base).
    NoWay,
    /// Very unlikely (15% base).
    VeryUnlikely,
    /// Somewhat unlikely (25% base).
    Unlikely,
    /// Even odds (50% base).
    FiftyFifty,
    /// Somewhat likely (65% base).
    SomewhatLikely,
    /// Probably (75% base).
    Likely,
    /// Very likely (85% base).
    VeryLikely,
    /// Almost certain (90% base).
    NearSureThing,
    /// Virtually guaranteed (95% base).
    ASureThing,
    /// Cannot fail (99% base).
    HasToBe,
}

impl Likelihood {
    /// Parse a likelihood from a user-supplied string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_'], " ").trim() {
            "impossible" => Some(Self::Impossible),
            "no way" | "noway" => Some(Self::NoWay),
            "very unlikely" => Some(Self::VeryUnlikely),
            "unlikely" => Some(Self::Unlikely),
            "50/50" | "5050" | "fifty fifty" | "even" => Some(Self::FiftyFifty),
            "somewhat likely" => Some(Self::SomewhatLikely),
            "likely" => Some(Self::Likely),
            "very likely" => Some(Self::VeryLikely),
            "near sure thing" | "near sure" => Some(Self::NearSureThing),
            "a sure thing" | "sure thing" | "sure" => Some(Self::ASureThing),
            "has to be" | "certain" => Some(Self::HasToBe),
            _ => None,
        }
    }

    /// All likelihood values in order from least to most likely.
    pub fn all() -> &'static [Self] {
        &[
            Self::Impossible,
            Self::NoWay,
            Self::VeryUnlikely,
            Self::Unlikely,
            Self::FiftyFifty,
            Self::SomewhatLikely,
            Self::Likely,
            Self::VeryLikely,
            Self::NearSureThing,
            Self::ASureThing,
            Self::HasToBe,
        ]
    }

    fn index(self) -> usize {
        match self {
            Self::Impossible => 0,
            Self::NoWay => 1,
            Self::VeryUnlikely => 2,
            Self::Unlikely => 3,
            Self::FiftyFifty => 4,
            Self::SomewhatLikely => 5,
            Self::Likely => 6,
            Self::VeryLikely => 7,
            Self::NearSureThing => 8,
            Self::ASureThing => 9,
            Self::HasToBe => 10,
        }
    }
}

impl std::fmt::Display for Likelihood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Impossible => write!(f, "Impossible"),
            Self::NoWay => write!(f, "No Way"),
            Self::VeryUnlikely => write!(f, "Very Unlikely"),
            Self::Unlikely => write!(f, "Unlikely"),
            Self::FiftyFifty => write!(f, "50/50"),
            Self::SomewhatLikely => write!(f, "Somewhat Likely"),
            Self::Likely => write!(f, "Likely"),
            Self::VeryLikely => write!(f, "Very Likely"),
            Self::NearSureThing => write!(f, "Near Sure Thing"),
            Self::ASureThing => write!(f, "A Sure Thing"),
            Self::HasToBe => write!(f, "Has To Be"),
        }
    }
}

/// The oracle's answer to a yes/no question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleAnswer {
    /// Strong yes — beyond what was expected.
    ExceptionalYes,
    /// Affirmative.
    Yes,
    /// Negative.
    No,
    /// Strong no — worse than expected.
    ExceptionalNo,
}

impl std::fmt::Display for OracleAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExceptionalYes => write!(f, "Exceptional Yes"),
            Self::Yes => write!(f, "Yes"),
            Self::No => write!(f, "No"),
            Self::ExceptionalNo => write!(f, "Exceptional No"),
        }
    }
}

/// The full result of an oracle consultation.
#[derive(Debug, Clone)]
pub struct OracleResult {
    /// The oracle's answer.
    pub answer: OracleAnswer,
    /// The d100 roll (1-100).
    pub roll: u32,
    /// The target threshold for "Yes".
    pub target: u32,
    /// A random event, if one was triggered.
    pub random_event: Option<RandomEvent>,
}

/// Fate chart threshold table: `FATE_CHART[likelihood_index][chaos - 1]`.
///
/// Each value is the d100 threshold at or below which the answer is "Yes".
/// Rows are likelihood (Impossible..HasToBe), columns are chaos (1..9).
const FATE_CHART: [[u32; 9]; 11] = [
    // chaos:  1   2   3   4   5   6   7   8   9
    [1, 2, 3, 5, 5, 5, 7, 10, 15],        // Impossible
    [2, 3, 5, 5, 5, 10, 15, 20, 25],      // No Way
    [3, 5, 7, 10, 15, 20, 25, 35, 45],    // Very Unlikely
    [5, 10, 15, 20, 25, 35, 45, 50, 55],  // Unlikely
    [10, 15, 25, 35, 50, 55, 65, 75, 85], // 50/50
    [15, 25, 35, 45, 55, 65, 75, 85, 90], // Somewhat Likely
    [20, 35, 45, 55, 65, 75, 85, 90, 95], // Likely
    [30, 45, 55, 65, 75, 85, 90, 95, 97], // Very Likely
    [40, 55, 65, 75, 85, 90, 95, 97, 99], // Near Sure Thing
    [50, 65, 75, 85, 90, 95, 97, 99, 99], // A Sure Thing
    [55, 75, 85, 90, 95, 97, 99, 99, 99], // Has To Be
];

/// Look up the "Yes" threshold for a given likelihood and chaos factor.
pub fn fate_threshold(likelihood: Likelihood, chaos: u32) -> u32 {
    let chaos_idx = (chaos.clamp(1, 9) - 1) as usize;
    FATE_CHART[likelihood.index()][chaos_idx]
}

/// Check if a roll triggers a random event.
///
/// A random event is triggered when the d100 roll has matching digits
/// (11, 22, 33, ..., 99) AND the matching digit is <= the chaos factor.
pub fn is_random_event_trigger(roll: u32, chaos: u32) -> bool {
    if !(11..=99).contains(&roll) {
        return false;
    }
    let tens = roll / 10;
    let ones = roll % 10;
    tens == ones && tens <= chaos
}

/// Consult the oracle with a yes/no question.
pub fn consult_oracle(
    likelihood: Likelihood,
    chaos: u32,
    rng: &mut StdRng,
    config: &OracleConfig,
) -> OracleResult {
    let target = fate_threshold(likelihood, chaos);
    let roll: u32 = rng.random_range(1..=100);

    // Determine base answer
    let is_yes = roll <= target;

    // Exceptional thresholds: extreme 1/5 of each range
    let exceptional_yes_threshold = target / 5;
    let exceptional_no_threshold = target + ((100 - target) * 4 / 5);

    let answer = if is_yes && roll <= exceptional_yes_threshold.max(1) {
        OracleAnswer::ExceptionalYes
    } else if is_yes {
        OracleAnswer::Yes
    } else if roll >= exceptional_no_threshold.min(100) {
        OracleAnswer::ExceptionalNo
    } else {
        OracleAnswer::No
    };

    // Check for random event
    let random_event = if is_random_event_trigger(roll, chaos) {
        Some(generate_random_event(rng, config))
    } else {
        None
    };

    OracleResult {
        answer,
        roll,
        target,
        random_event,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn fate_threshold_increases_with_chaos() {
        for lk in Likelihood::all() {
            let mut prev = 0;
            for chaos in 1..=9 {
                let t = fate_threshold(*lk, chaos);
                assert!(t >= prev, "{lk} chaos {chaos}: {t} < {prev}");
                prev = t;
            }
        }
    }

    #[test]
    fn fate_threshold_increases_with_likelihood() {
        for chaos in 1..=9 {
            let mut prev = 0;
            for lk in Likelihood::all() {
                let t = fate_threshold(*lk, chaos);
                assert!(t >= prev, "{lk} chaos {chaos}: {t} < {prev}");
                prev = t;
            }
        }
    }

    #[test]
    fn fate_threshold_bounds() {
        // Minimum: Impossible at chaos 1
        assert!(fate_threshold(Likelihood::Impossible, 1) >= 1);
        // Maximum: HasToBe at chaos 9
        assert!(fate_threshold(Likelihood::HasToBe, 9) <= 99);
    }

    #[test]
    fn random_event_trigger_on_doubles() {
        assert!(is_random_event_trigger(11, 1));
        assert!(is_random_event_trigger(22, 2));
        assert!(is_random_event_trigger(55, 5));
        assert!(is_random_event_trigger(99, 9));
    }

    #[test]
    fn random_event_not_triggered_above_chaos() {
        assert!(!is_random_event_trigger(33, 2)); // 3 > chaos 2
        assert!(!is_random_event_trigger(66, 5)); // 6 > chaos 5
    }

    #[test]
    fn random_event_not_on_non_doubles() {
        assert!(!is_random_event_trigger(12, 9));
        assert!(!is_random_event_trigger(50, 9));
        assert!(!is_random_event_trigger(73, 9));
    }

    #[test]
    fn random_event_edge_cases() {
        // Single digits and 100 never trigger
        assert!(!is_random_event_trigger(1, 9));
        assert!(!is_random_event_trigger(5, 9));
        assert!(!is_random_event_trigger(100, 9));
    }

    #[test]
    fn consult_oracle_deterministic() {
        let config = OracleConfig::default();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);
        let r1 = consult_oracle(Likelihood::FiftyFifty, 5, &mut rng1, &config);
        let r2 = consult_oracle(Likelihood::FiftyFifty, 5, &mut rng2, &config);
        assert_eq!(r1.roll, r2.roll);
        assert_eq!(r1.answer, r2.answer);
    }

    #[test]
    fn consult_oracle_always_valid() {
        let config = OracleConfig::default();
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..200 {
            for lk in Likelihood::all() {
                for chaos in 1..=9 {
                    let r = consult_oracle(*lk, chaos, &mut rng, &config);
                    assert!((1..=100).contains(&r.roll));
                    assert!(r.target >= 1 && r.target <= 99);
                }
            }
        }
    }

    #[test]
    fn likelihood_parse_variants() {
        assert_eq!(Likelihood::parse("likely"), Some(Likelihood::Likely));
        assert_eq!(Likelihood::parse("50/50"), Some(Likelihood::FiftyFifty));
        assert_eq!(Likelihood::parse("UNLIKELY"), Some(Likelihood::Unlikely));
        assert_eq!(
            Likelihood::parse("very-likely"),
            Some(Likelihood::VeryLikely)
        );
        assert_eq!(
            Likelihood::parse("sure thing"),
            Some(Likelihood::ASureThing)
        );
        assert_eq!(Likelihood::parse("gibberish"), None);
    }

    #[test]
    fn likelihood_display() {
        assert_eq!(Likelihood::FiftyFifty.to_string(), "50/50");
        assert_eq!(Likelihood::VeryLikely.to_string(), "Very Likely");
        assert_eq!(Likelihood::HasToBe.to_string(), "Has To Be");
    }

    #[test]
    fn oracle_answer_display() {
        assert_eq!(OracleAnswer::ExceptionalYes.to_string(), "Exceptional Yes");
        assert_eq!(OracleAnswer::No.to_string(), "No");
    }
}
