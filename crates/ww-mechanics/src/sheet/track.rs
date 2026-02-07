//! Resource tracks (HP, Ruin, Honor, Momentum, etc.).
//!
//! A track is a clamped numeric value with a min and max, used to
//! represent mutable character resources across different game systems.

use serde::{Deserialize, Serialize};

/// A named numeric resource that is clamped between min and max.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Display name of the track.
    pub name: String,
    /// Current value.
    pub current: i32,
    /// Maximum value.
    pub max: i32,
    /// Minimum value (usually 0).
    pub min: i32,
}

impl Track {
    /// Create a new track starting at its maximum value.
    pub fn new(name: impl Into<String>, max: i32) -> Self {
        Self {
            name: name.into(),
            current: max,
            max,
            min: 0,
        }
    }

    /// Create a new track with a custom minimum and starting value.
    pub fn with_range(name: impl Into<String>, current: i32, min: i32, max: i32) -> Self {
        let clamped = current.clamp(min, max);
        Self {
            name: name.into(),
            current: clamped,
            max,
            min,
        }
    }

    /// Adjust the track by a delta, clamping to bounds. Returns the new value.
    pub fn adjust(&mut self, delta: i32) -> i32 {
        self.current = (self.current + delta).clamp(self.min, self.max);
        self.current
    }

    /// Returns true if the track is at its minimum value.
    pub fn is_empty(&self) -> bool {
        self.current <= self.min
    }

    /// Returns true if the track is at its maximum value.
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Returns the fraction of the track that is filled (0.0 to 1.0).
    pub fn fraction(&self) -> f64 {
        let range = self.max - self.min;
        if range == 0 {
            return 1.0;
        }
        (self.current - self.min) as f64 / range as f64
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}/{}", self.name, self.current, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_max() {
        let t = Track::new("HP", 10);
        assert_eq!(t.current, 10);
        assert_eq!(t.max, 10);
        assert_eq!(t.min, 0);
        assert!(t.is_full());
        assert!(!t.is_empty());
    }

    #[test]
    fn with_range() {
        let t = Track::with_range("Ruin", 1, 1, 6);
        assert_eq!(t.current, 1);
        assert_eq!(t.min, 1);
        assert_eq!(t.max, 6);
    }

    #[test]
    fn adjust_clamps_to_max() {
        let mut t = Track::new("HP", 5);
        assert_eq!(t.adjust(10), 5);
        assert!(t.is_full());
    }

    #[test]
    fn adjust_clamps_to_min() {
        let mut t = Track::new("HP", 5);
        assert_eq!(t.adjust(-20), 0);
        assert!(t.is_empty());
    }

    #[test]
    fn adjust_normal() {
        let mut t = Track::new("HP", 10);
        assert_eq!(t.adjust(-3), 7);
        assert!(!t.is_empty());
        assert!(!t.is_full());
    }

    #[test]
    fn fraction() {
        let mut t = Track::new("HP", 10);
        assert!((t.fraction() - 1.0).abs() < f64::EPSILON);
        t.adjust(-5);
        assert!((t.fraction() - 0.5).abs() < f64::EPSILON);
        t.adjust(-5);
        assert!((t.fraction()).abs() < f64::EPSILON);
    }

    #[test]
    fn fraction_zero_range() {
        let t = Track::with_range("Fixed", 5, 5, 5);
        assert!((t.fraction() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn display() {
        let t = Track::new("Stress", 5);
        assert_eq!(t.to_string(), "Stress: 5/5");
    }

    #[test]
    fn with_range_clamps_initial() {
        let t = Track::with_range("Test", 100, 0, 10);
        assert_eq!(t.current, 10);
    }
}
