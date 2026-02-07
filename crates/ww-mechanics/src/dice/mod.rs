//! Dice types, pools, and rolling.
//!
//! Supports standard polyhedral dice (d4 through d100) and custom dice.
//! Dice can be tagged (light, dark, momentum, wagered) for system-specific
//! behavior during resolution.

pub mod pool;
pub mod roll;

pub use pool::DicePool;
pub use roll::{DieResult, RollResult};

use serde::{Deserialize, Serialize};

/// A polyhedral die type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Die {
    /// Four-sided die.
    D4,
    /// Six-sided die.
    D6,
    /// Eight-sided die.
    D8,
    /// Ten-sided die.
    D10,
    /// Twelve-sided die.
    D12,
    /// Twenty-sided die.
    D20,
    /// Percentile die (1-100).
    D100,
    /// A die with a custom number of sides.
    Custom(u32),
}

impl Die {
    /// Returns the number of sides on this die.
    pub fn sides(self) -> u32 {
        match self {
            Self::D4 => 4,
            Self::D6 => 6,
            Self::D8 => 8,
            Self::D10 => 10,
            Self::D12 => 12,
            Self::D20 => 20,
            Self::D100 => 100,
            Self::Custom(n) => n,
        }
    }

    /// Parse a die from a string like "d20", "d6", "d100".
    pub fn from_str_tag(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "d4" => Some(Self::D4),
            "d6" => Some(Self::D6),
            "d8" => Some(Self::D8),
            "d10" => Some(Self::D10),
            "d12" => Some(Self::D12),
            "d20" => Some(Self::D20),
            "d100" => Some(Self::D100),
            other => {
                let num = other.strip_prefix('d')?.parse::<u32>().ok()?;
                if num >= 2 {
                    Some(Self::Custom(num))
                } else {
                    None
                }
            }
        }
    }
}

impl std::fmt::Display for Die {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::D4 => write!(f, "d4"),
            Self::D6 => write!(f, "d6"),
            Self::D8 => write!(f, "d8"),
            Self::D10 => write!(f, "d10"),
            Self::D12 => write!(f, "d12"),
            Self::D20 => write!(f, "d20"),
            Self::D100 => write!(f, "d100"),
            Self::Custom(n) => write!(f, "d{n}"),
        }
    }
}

/// A tag applied to a die for system-specific behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DiceTag {
    /// No special tag — a standard die.
    #[default]
    Default,
    /// A light die (Trophy Gold: belongs to the player).
    Light,
    /// A dark die (Trophy Gold: risk of Ruin).
    Dark,
    /// A momentum die (2d20: spent from momentum pool).
    Momentum,
    /// A wagered die (Blood & Honor: staked on the outcome).
    Wagered,
    /// A custom tag for user-defined systems.
    Custom(String),
}

impl std::fmt::Display for DiceTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
            Self::Momentum => write!(f, "momentum"),
            Self::Wagered => write!(f, "wagered"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_sides() {
        assert_eq!(Die::D4.sides(), 4);
        assert_eq!(Die::D6.sides(), 6);
        assert_eq!(Die::D8.sides(), 8);
        assert_eq!(Die::D10.sides(), 10);
        assert_eq!(Die::D12.sides(), 12);
        assert_eq!(Die::D20.sides(), 20);
        assert_eq!(Die::D100.sides(), 100);
        assert_eq!(Die::Custom(30).sides(), 30);
    }

    #[test]
    fn die_from_str() {
        assert_eq!(Die::from_str_tag("d20"), Some(Die::D20));
        assert_eq!(Die::from_str_tag("D6"), Some(Die::D6));
        assert_eq!(Die::from_str_tag("d100"), Some(Die::D100));
        assert_eq!(Die::from_str_tag("d30"), Some(Die::Custom(30)));
        assert_eq!(Die::from_str_tag("d1"), None);
        assert_eq!(Die::from_str_tag("foo"), None);
    }

    #[test]
    fn die_display() {
        assert_eq!(Die::D20.to_string(), "d20");
        assert_eq!(Die::Custom(30).to_string(), "d30");
    }

    #[test]
    fn dice_tag_display() {
        assert_eq!(DiceTag::Default.to_string(), "default");
        assert_eq!(DiceTag::Light.to_string(), "light");
        assert_eq!(DiceTag::Dark.to_string(), "dark");
        assert_eq!(DiceTag::Custom("hero".to_string()).to_string(), "hero");
    }
}
