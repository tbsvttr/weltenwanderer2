//! Dice roll results and aggregation.

use serde::{Deserialize, Serialize};

use super::{DiceTag, Die};

/// The result of rolling a single die.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieResult {
    /// The type of die that was rolled.
    pub die: Die,
    /// The tag on this die.
    pub tag: DiceTag,
    /// The value rolled (1 to die.sides()).
    pub value: u32,
}

/// The result of rolling an entire dice pool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RollResult {
    /// Individual die results.
    pub dice: Vec<DieResult>,
}

impl RollResult {
    /// Sum of all die values.
    pub fn total(&self) -> u32 {
        self.dice.iter().map(|d| d.value).sum()
    }

    /// The highest single die value, or 0 if empty.
    pub fn highest(&self) -> u32 {
        self.dice.iter().map(|d| d.value).max().unwrap_or(0)
    }

    /// The lowest single die value, or 0 if empty.
    pub fn lowest(&self) -> u32 {
        self.dice.iter().map(|d| d.value).min().unwrap_or(0)
    }

    /// Count dice with values at or below the given threshold.
    pub fn count_at_or_below(&self, threshold: u32) -> u32 {
        self.dice.iter().filter(|d| d.value <= threshold).count() as u32
    }

    /// Count dice with values at or above the given threshold.
    pub fn count_at_or_above(&self, threshold: u32) -> u32 {
        self.dice.iter().filter(|d| d.value >= threshold).count() as u32
    }

    /// Get all die results with a specific tag.
    pub fn by_tag(&self, tag: &DiceTag) -> Vec<&DieResult> {
        self.dice.iter().filter(|d| &d.tag == tag).collect()
    }

    /// The highest value among dice with a specific tag, or 0 if none.
    pub fn highest_by_tag(&self, tag: &DiceTag) -> u32 {
        self.by_tag(tag).iter().map(|d| d.value).max().unwrap_or(0)
    }

    /// Number of dice in the result.
    pub fn count(&self) -> usize {
        self.dice.len()
    }
}

impl std::fmt::Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values: Vec<String> = self.dice.iter().map(|d| d.value.to_string()).collect();
        write!(f, "[{}] = {}", values.join(", "), self.total())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(values: &[(Die, DiceTag, u32)]) -> RollResult {
        RollResult {
            dice: values
                .iter()
                .map(|(die, tag, value)| DieResult {
                    die: *die,
                    tag: tag.clone(),
                    value: *value,
                })
                .collect(),
        }
    }

    #[test]
    fn total() {
        let r = make_result(&[
            (Die::D20, DiceTag::Default, 15),
            (Die::D20, DiceTag::Default, 8),
        ]);
        assert_eq!(r.total(), 23);
    }

    #[test]
    fn highest_and_lowest() {
        let r = make_result(&[
            (Die::D6, DiceTag::Default, 3),
            (Die::D6, DiceTag::Default, 6),
            (Die::D6, DiceTag::Default, 1),
        ]);
        assert_eq!(r.highest(), 6);
        assert_eq!(r.lowest(), 1);
    }

    #[test]
    fn empty_result() {
        let r = RollResult::default();
        assert_eq!(r.total(), 0);
        assert_eq!(r.highest(), 0);
        assert_eq!(r.lowest(), 0);
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn count_at_or_below() {
        let r = make_result(&[
            (Die::D20, DiceTag::Default, 5),
            (Die::D20, DiceTag::Default, 10),
            (Die::D20, DiceTag::Default, 15),
        ]);
        assert_eq!(r.count_at_or_below(10), 2);
        assert_eq!(r.count_at_or_below(4), 0);
        assert_eq!(r.count_at_or_below(20), 3);
    }

    #[test]
    fn count_at_or_above() {
        let r = make_result(&[
            (Die::D6, DiceTag::Default, 2),
            (Die::D6, DiceTag::Default, 4),
            (Die::D6, DiceTag::Default, 6),
        ]);
        assert_eq!(r.count_at_or_above(4), 2);
        assert_eq!(r.count_at_or_above(6), 1);
        assert_eq!(r.count_at_or_above(1), 3);
    }

    #[test]
    fn by_tag() {
        let r = make_result(&[
            (Die::D6, DiceTag::Light, 4),
            (Die::D6, DiceTag::Dark, 6),
            (Die::D6, DiceTag::Light, 2),
        ]);
        assert_eq!(r.by_tag(&DiceTag::Light).len(), 2);
        assert_eq!(r.by_tag(&DiceTag::Dark).len(), 1);
        assert_eq!(r.highest_by_tag(&DiceTag::Light), 4);
        assert_eq!(r.highest_by_tag(&DiceTag::Dark), 6);
    }

    #[test]
    fn display() {
        let r = make_result(&[
            (Die::D6, DiceTag::Default, 3),
            (Die::D6, DiceTag::Default, 5),
        ]);
        assert_eq!(r.to_string(), "[3, 5] = 8");
    }
}
