//! Dice pool construction and rolling.

use rand::Rng;
use rand::rngs::StdRng;

use super::roll::{DieResult, RollResult};
use super::{DiceTag, Die};

/// A collection of dice to be rolled together.
#[derive(Debug, Clone, Default)]
pub struct DicePool {
    /// The dice in this pool with their tags.
    pub dice: Vec<(Die, DiceTag)>,
}

impl DicePool {
    /// Create an empty dice pool.
    pub fn new() -> Self {
        Self { dice: Vec::new() }
    }

    /// Add `count` dice of the given type with the default tag.
    pub fn add(mut self, die: Die, count: u32) -> Self {
        for _ in 0..count {
            self.dice.push((die, DiceTag::Default));
        }
        self
    }

    /// Add `count` dice of the given type with a specific tag.
    pub fn add_tagged(mut self, die: Die, tag: DiceTag, count: u32) -> Self {
        for _ in 0..count {
            self.dice.push((die, tag.clone()));
        }
        self
    }

    /// Returns how many dice are in the pool.
    pub fn count(&self) -> usize {
        self.dice.len()
    }

    /// Returns true if the pool has no dice.
    pub fn is_empty(&self) -> bool {
        self.dice.is_empty()
    }

    /// Roll all dice in the pool using the given RNG.
    pub fn roll(&self, rng: &mut StdRng) -> RollResult {
        let dice = self
            .dice
            .iter()
            .map(|(die, tag)| {
                let value = rng.random_range(1..=die.sides());
                DieResult {
                    die: *die,
                    tag: tag.clone(),
                    value,
                }
            })
            .collect();
        RollResult { dice }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn empty_pool() {
        let pool = DicePool::new();
        assert_eq!(pool.count(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn add_dice() {
        let pool = DicePool::new().add(Die::D20, 2).add(Die::D6, 3);
        assert_eq!(pool.count(), 5);
        assert!(!pool.is_empty());
    }

    #[test]
    fn add_tagged_dice() {
        let pool = DicePool::new()
            .add_tagged(Die::D6, DiceTag::Light, 2)
            .add_tagged(Die::D6, DiceTag::Dark, 1);
        assert_eq!(pool.count(), 3);
        assert_eq!(pool.dice[0].1, DiceTag::Light);
        assert_eq!(pool.dice[2].1, DiceTag::Dark);
    }

    #[test]
    fn roll_produces_valid_values() {
        let mut rng = StdRng::seed_from_u64(42);
        let pool = DicePool::new().add(Die::D6, 10);
        let result = pool.roll(&mut rng);
        assert_eq!(result.dice.len(), 10);
        for die_result in &result.dice {
            assert!((1..=6).contains(&die_result.value));
        }
    }

    #[test]
    fn roll_deterministic_with_seed() {
        let pool = DicePool::new().add(Die::D20, 3);
        let mut rng1 = StdRng::seed_from_u64(99);
        let mut rng2 = StdRng::seed_from_u64(99);
        let r1 = pool.roll(&mut rng1);
        let r2 = pool.roll(&mut rng2);
        for (a, b) in r1.dice.iter().zip(r2.dice.iter()) {
            assert_eq!(a.value, b.value);
        }
    }
}
