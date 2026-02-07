//! NPC reaction rolls.
//!
//! A 2d10 roll determines an NPC's attitude on a 7-level scale
//! from Hostile to Generous.

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

/// NPC reaction level on a 7-point scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NpcReaction {
    /// Actively antagonistic.
    Hostile,
    /// Uncooperative and cold.
    Unfriendly,
    /// Wary and guarded.
    Cautious,
    /// Indifferent.
    Neutral,
    /// Pleasantly engaged.
    Sociable,
    /// Actively helpful.
    Friendly,
    /// Exceptionally giving.
    Generous,
}

impl std::fmt::Display for NpcReaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hostile => write!(f, "Hostile"),
            Self::Unfriendly => write!(f, "Unfriendly"),
            Self::Cautious => write!(f, "Cautious"),
            Self::Neutral => write!(f, "Neutral"),
            Self::Sociable => write!(f, "Sociable"),
            Self::Friendly => write!(f, "Friendly"),
            Self::Generous => write!(f, "Generous"),
        }
    }
}

/// Result of an NPC reaction roll.
#[derive(Debug, Clone)]
pub struct ReactionResult {
    /// The reaction level.
    pub reaction: NpcReaction,
    /// The 2d10 roll total (2-20).
    pub roll: u32,
}

/// Roll an NPC reaction (2d10 → 2-20 mapped to 7 levels).
pub fn roll_npc_reaction(rng: &mut StdRng) -> ReactionResult {
    let d1: u32 = rng.random_range(1..=10);
    let d2: u32 = rng.random_range(1..=10);
    let roll = d1 + d2;

    let reaction = match roll {
        2..=3 => NpcReaction::Hostile,
        4..=5 => NpcReaction::Unfriendly,
        6..=8 => NpcReaction::Cautious,
        9..=12 => NpcReaction::Neutral,
        13..=15 => NpcReaction::Sociable,
        16..=18 => NpcReaction::Friendly,
        19..=20 => NpcReaction::Generous,
        _ => unreachable!(),
    };

    ReactionResult { reaction, roll }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn reaction_roll_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..500 {
            let r = roll_npc_reaction(&mut rng);
            assert!((2..=20).contains(&r.roll));
        }
    }

    #[test]
    fn all_reactions_reachable() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..5000 {
            let r = roll_npc_reaction(&mut rng);
            seen.insert(format!("{:?}", r.reaction));
        }
        assert_eq!(seen.len(), 7, "missing reactions: {seen:?}");
    }

    #[test]
    fn reaction_ordering() {
        assert!(NpcReaction::Hostile < NpcReaction::Unfriendly);
        assert!(NpcReaction::Neutral < NpcReaction::Sociable);
        assert!(NpcReaction::Friendly < NpcReaction::Generous);
    }

    #[test]
    fn reaction_display() {
        assert_eq!(NpcReaction::Hostile.to_string(), "Hostile");
        assert_eq!(NpcReaction::Neutral.to_string(), "Neutral");
        assert_eq!(NpcReaction::Generous.to_string(), "Generous");
    }

    #[test]
    fn reaction_serde_roundtrip() {
        let json = serde_json::to_string(&NpcReaction::Friendly).unwrap();
        let r: NpcReaction = serde_json::from_str(&json).unwrap();
        assert_eq!(r, NpcReaction::Friendly);
    }
}
