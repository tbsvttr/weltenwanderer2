//! Scene management for solo TTRPG play.
//!
//! Each scene has a setup, a chaos check that may alter or interrupt it,
//! and a summary when it ends. The chaos factor adjusts based on how
//! the scene went for the player character.

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::oracle::event::{RandomEvent, generate_random_event};
use crate::oracle::tables::OracleConfig;

/// Status of a scene after the chaos check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneStatus {
    /// Scene proceeds as the player expected.
    Normal,
    /// Scene is altered — something different happens.
    Altered,
    /// Scene is interrupted by a random event replacing the setup.
    Interrupted(RandomEvent),
}

impl std::fmt::Display for SceneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Altered => write!(f, "Altered"),
            Self::Interrupted(event) => write!(f, "Interrupted ({event})"),
        }
    }
}

/// A scene in the solo session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Scene number (1-based).
    pub number: u32,
    /// What the player expected to happen.
    pub setup: String,
    /// Scene status after chaos check.
    pub status: SceneStatus,
    /// Optional summary written at scene end.
    pub summary: Option<String>,
    /// When the scene started.
    pub started_at: DateTime<Utc>,
    /// When the scene ended (if completed).
    pub ended_at: Option<DateTime<Utc>>,
}

/// Check a scene setup against the chaos factor.
///
/// Roll d10: if the roll is <= chaos, the scene is modified.
/// Odd roll = altered, even roll = interrupted (random event replaces setup).
pub fn check_scene_setup(chaos: u32, rng: &mut StdRng, config: &OracleConfig) -> SceneStatus {
    let roll: u32 = rng.random_range(1..=10);
    if roll <= chaos {
        if roll % 2 == 1 {
            SceneStatus::Altered
        } else {
            SceneStatus::Interrupted(generate_random_event(rng, config))
        }
    } else {
        SceneStatus::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn chaos_1_rarely_modifies() {
        let config = OracleConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let mut modified = 0;
        let trials = 1000;
        for _ in 0..trials {
            match check_scene_setup(1, &mut rng, &config) {
                SceneStatus::Normal => {}
                _ => modified += 1,
            }
        }
        // Chaos 1: ~10% chance (roll 1 on d10)
        assert!(modified < 200, "too many modifications: {modified}/1000");
    }

    #[test]
    fn chaos_9_often_modifies() {
        let config = OracleConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        let mut modified = 0;
        let trials = 1000;
        for _ in 0..trials {
            match check_scene_setup(9, &mut rng, &config) {
                SceneStatus::Normal => {}
                _ => modified += 1,
            }
        }
        // Chaos 9: ~90% chance (rolls 1-9 on d10)
        assert!(modified > 700, "too few modifications: {modified}/1000");
    }

    #[test]
    fn interrupted_scenes_have_events() {
        let config = OracleConfig::default();
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..500 {
            if let SceneStatus::Interrupted(event) = check_scene_setup(9, &mut rng, &config) {
                assert!(!event.action.is_empty());
                assert!(!event.subject.is_empty());
                return;
            }
        }
        panic!("no interrupted scenes generated in 500 tries");
    }

    #[test]
    fn scene_status_display() {
        assert_eq!(SceneStatus::Normal.to_string(), "Normal");
        assert_eq!(SceneStatus::Altered.to_string(), "Altered");
    }
}
