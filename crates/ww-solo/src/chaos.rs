//! Chaos factor tracking for the solo engine.
//!
//! The chaos factor (1-9) represents how unpredictable the story has become.
//! Higher chaos increases the chance of "Yes" answers from the oracle and
//! makes scene interruptions more likely.

use serde::{Deserialize, Serialize};

/// The chaos factor, ranging from 1 (orderly) to 9 (chaotic).
///
/// Starts at 5 by default. Increases when scenes go badly for the player,
/// decreases when they go well.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosFactor {
    value: u32,
}

impl ChaosFactor {
    /// Create a new chaos factor, clamped to 1-9.
    pub fn new(value: u32) -> Self {
        Self {
            value: value.clamp(1, 9),
        }
    }

    /// Get the current chaos value.
    pub fn value(&self) -> u32 {
        self.value
    }

    /// Increase chaos by 1 (max 9). Called when a scene goes badly.
    pub fn increase(&mut self) {
        self.value = (self.value + 1).min(9);
    }

    /// Decrease chaos by 1 (min 1). Called when a scene goes well.
    pub fn decrease(&mut self) {
        self.value = self.value.saturating_sub(1).max(1);
    }
}

impl Default for ChaosFactor {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_five() {
        assert_eq!(ChaosFactor::default().value(), 5);
    }

    #[test]
    fn clamped_on_creation() {
        assert_eq!(ChaosFactor::new(0).value(), 1);
        assert_eq!(ChaosFactor::new(100).value(), 9);
        assert_eq!(ChaosFactor::new(5).value(), 5);
    }

    #[test]
    fn increase_caps_at_nine() {
        let mut c = ChaosFactor::new(8);
        c.increase();
        assert_eq!(c.value(), 9);
        c.increase();
        assert_eq!(c.value(), 9);
    }

    #[test]
    fn decrease_floors_at_one() {
        let mut c = ChaosFactor::new(2);
        c.decrease();
        assert_eq!(c.value(), 1);
        c.decrease();
        assert_eq!(c.value(), 1);
    }

    #[test]
    fn round_trip_serde() {
        let c = ChaosFactor::new(7);
        let json = serde_json::to_string(&c).unwrap();
        let c2: ChaosFactor = serde_json::from_str(&json).unwrap();
        assert_eq!(c2.value(), 7);
    }
}
