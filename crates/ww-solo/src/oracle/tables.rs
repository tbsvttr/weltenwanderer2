//! Meaning tables for random event interpretation.
//!
//! Action and subject word lists used by the random event generator
//! to create event descriptions that the player interprets narratively.

use rand::Rng;
use rand::rngs::StdRng;

use ww_core::World;
use ww_core::entity::MetadataValue;

/// Action/verb words for event meaning (100 entries).
pub const ACTION_WORDS: &[&str] = &[
    "Attainment",
    "Starting",
    "Neglect",
    "Fight",
    "Recruit",
    "Triumph",
    "Violate",
    "Oppose",
    "Malice",
    "Communicate",
    "Persecute",
    "Increase",
    "Decrease",
    "Abandon",
    "Gratify",
    "Inquire",
    "Antagonize",
    "Move",
    "Waste",
    "Truce",
    "Release",
    "Befriend",
    "Judge",
    "Desert",
    "Dominate",
    "Procrastinate",
    "Praise",
    "Separate",
    "Take",
    "Break",
    "Heal",
    "Delay",
    "Stop",
    "Lie",
    "Return",
    "Imitate",
    "Struggle",
    "Inform",
    "Bestow",
    "Postpone",
    "Expose",
    "Haggle",
    "Imprison",
    "Release",
    "Celebrate",
    "Develop",
    "Travel",
    "Block",
    "Harm",
    "Debase",
    "Overindulge",
    "Adjourn",
    "Adversity",
    "Kill",
    "Disrupt",
    "Usurp",
    "Create",
    "Betray",
    "Agree",
    "Abuse",
    "Oppress",
    "Inspect",
    "Ambush",
    "Spy",
    "Attach",
    "Carry",
    "Open",
    "Carelessness",
    "Ruin",
    "Extravagance",
    "Trick",
    "Arrive",
    "Propose",
    "Divide",
    "Refuse",
    "Mistrust",
    "Deceive",
    "Cruelty",
    "Intolerance",
    "Trust",
    "Excitement",
    "Activity",
    "Assist",
    "Care",
    "Negligence",
    "Passion",
    "Work",
    "Control",
    "Attract",
    "Failure",
    "Pursue",
    "Vengeance",
    "Proceedings",
    "Dispute",
    "Punish",
    "Guide",
    "Transform",
    "Overthrow",
    "Oppress",
    "Change",
];

/// Subject/noun words for event meaning (100 entries).
pub const SUBJECT_WORDS: &[&str] = &[
    "Goals",
    "Dreams",
    "Environment",
    "Outside",
    "Inside",
    "Reality",
    "Allies",
    "Enemies",
    "Evil",
    "Good",
    "Emotions",
    "Opposition",
    "War",
    "Peace",
    "Innocent",
    "Love",
    "Spirit",
    "Intellect",
    "Ideas",
    "Joy",
    "Evidence",
    "Burden",
    "Jealousy",
    "Dispute",
    "Home",
    "Investment",
    "Suffering",
    "Plans",
    "Lies",
    "Expectations",
    "Legal",
    "Bureaucracy",
    "Business",
    "Path",
    "News",
    "Exterior",
    "Advice",
    "Plot",
    "Competition",
    "Prison",
    "Illness",
    "Food",
    "Attention",
    "Success",
    "Failure",
    "Travel",
    "Jealousy",
    "Dispute",
    "Death",
    "Disruption",
    "Power",
    "Burden",
    "Intrigues",
    "Rumor",
    "Wounds",
    "Extravagance",
    "Representation",
    "Fame",
    "Anger",
    "Information",
    "Technology",
    "Weaponry",
    "Balance",
    "Mystical",
    "Military",
    "Riches",
    "Status",
    "Poverty",
    "Lies",
    "Vehicle",
    "Art",
    "Victory",
    "Dispute",
    "Elements",
    "Nature",
    "Animals",
    "Weather",
    "Masses",
    "Leadership",
    "Fears",
    "Danger",
    "Corruption",
    "Freedom",
    "Weapon",
    "Mundane",
    "Trial",
    "Energy",
    "Friendship",
    "Physical",
    "Benefits",
    "Tactics",
    "Allies",
    "Ambush",
    "Tension",
    "Direction",
    "Advantage",
    "Possessions",
    "Pain",
    "Wishes",
    "Tactics",
];

/// Pick a random action word.
pub fn random_action(rng: &mut StdRng) -> &'static str {
    ACTION_WORDS[rng.random_range(0..ACTION_WORDS.len())]
}

/// Pick a random subject word.
pub fn random_subject(rng: &mut StdRng) -> &'static str {
    SUBJECT_WORDS[rng.random_range(0..SUBJECT_WORDS.len())]
}

/// Oracle mode: action/subject tables (Mythic-style) or symbol list (Semiotic Standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleMode {
    /// Action + Subject word pairs (Mythic GME style).
    ActionSubject,
    /// Single symbol list (Semiotic Standard style, e.g. TEL).
    Symbols,
}

/// Oracle configuration loaded from world data or defaults.
///
/// Custom action/subject word lists can be defined in `.ww` files using an
/// `oracle` block:
///
/// ```text
/// lore "Oracle Tables" {
///     oracle {
///         actions ["Investigate", "Create", "Destroy"]
///         subjects ["Magic", "Weapon", "Enemy"]
///     }
/// }
/// ```
///
/// Or a single symbol list (Semiotic Standard):
///
/// ```text
/// lore "Oracle Tables" {
///     oracle {
///         symbols ["Anchor", "Atom", "Bird", ...]
///     }
/// }
/// ```
pub struct OracleConfig {
    /// Oracle mode: action/subject or symbols.
    pub mode: OracleMode,
    /// Action/verb words for random event generation (Mythic mode).
    pub actions: Vec<String>,
    /// Subject/noun words for random event generation (Mythic mode).
    pub subjects: Vec<String>,
    /// Symbol words for prompts (Semiotic Standard mode).
    pub symbols: Vec<String>,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            mode: OracleMode::ActionSubject,
            actions: ACTION_WORDS.iter().map(|s| (*s).to_string()).collect(),
            subjects: SUBJECT_WORDS.iter().map(|s| (*s).to_string()).collect(),
            symbols: Vec::new(),
        }
    }
}

impl OracleConfig {
    /// Load oracle configuration from a compiled world.
    ///
    /// Scans all entities for `oracle.actions`, `oracle.subjects`, or `oracle.symbols`
    /// properties (set via `oracle { ... }` blocks in `.ww` files).
    ///
    /// If `oracle.symbols` is found, uses Semiotic Standard mode (single symbol list).
    /// Otherwise, uses action/subject mode (Mythic GME style).
    ///
    /// Falls back to built-in defaults if not found or if custom lists are empty.
    pub fn from_world(world: &World) -> Self {
        let mut custom_actions: Option<Vec<String>> = None;
        let mut custom_subjects: Option<Vec<String>> = None;
        let mut custom_symbols: Option<Vec<String>> = None;

        for entity in world.all_entities() {
            if let Some(MetadataValue::List(items)) = entity.properties.get("oracle.actions") {
                let strings: Vec<String> = items
                    .iter()
                    .filter_map(|v| match v {
                        MetadataValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                if !strings.is_empty() {
                    custom_actions = Some(strings);
                }
            }
            if let Some(MetadataValue::List(items)) = entity.properties.get("oracle.subjects") {
                let strings: Vec<String> = items
                    .iter()
                    .filter_map(|v| match v {
                        MetadataValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                if !strings.is_empty() {
                    custom_subjects = Some(strings);
                }
            }
            if let Some(MetadataValue::List(items)) = entity.properties.get("oracle.symbols") {
                let strings: Vec<String> = items
                    .iter()
                    .filter_map(|v| match v {
                        MetadataValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                if !strings.is_empty() {
                    custom_symbols = Some(strings);
                }
            }
        }

        let defaults = Self::default();

        // If symbols are provided, use Semiotic Standard mode
        if let Some(symbols) = custom_symbols {
            Self {
                mode: OracleMode::Symbols,
                actions: Vec::new(),
                subjects: Vec::new(),
                symbols,
            }
        } else {
            // Otherwise use action/subject mode
            Self {
                mode: OracleMode::ActionSubject,
                actions: custom_actions.unwrap_or(defaults.actions),
                subjects: custom_subjects.unwrap_or(defaults.subjects),
                symbols: Vec::new(),
            }
        }
    }

    /// Pick a random action word from this config (ActionSubject mode).
    pub fn random_action<'a>(&'a self, rng: &mut StdRng) -> &'a str {
        if self.actions.is_empty() {
            ""
        } else {
            &self.actions[rng.random_range(0..self.actions.len())]
        }
    }

    /// Pick a random subject word from this config (ActionSubject mode).
    pub fn random_subject<'a>(&'a self, rng: &mut StdRng) -> &'a str {
        if self.subjects.is_empty() {
            ""
        } else {
            &self.subjects[rng.random_range(0..self.subjects.len())]
        }
    }

    /// Pick a random symbol from this config (Symbols mode).
    pub fn random_symbol<'a>(&'a self, rng: &mut StdRng) -> &'a str {
        if self.symbols.is_empty() {
            ""
        } else {
            &self.symbols[rng.random_range(0..self.symbols.len())]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use ww_core::{Entity, EntityKind, WorldMeta};

    #[test]
    fn tables_have_100_entries() {
        assert_eq!(ACTION_WORDS.len(), 100);
        assert_eq!(SUBJECT_WORDS.len(), 100);
    }

    #[test]
    fn random_picks_are_valid() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let a = random_action(&mut rng);
            let s = random_subject(&mut rng);
            assert!(!a.is_empty());
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn oracle_config_default_has_100_entries() {
        let config = OracleConfig::default();
        assert_eq!(config.mode, OracleMode::ActionSubject);
        assert_eq!(config.actions.len(), 100);
        assert_eq!(config.subjects.len(), 100);
        assert!(config.symbols.is_empty());
    }

    #[test]
    fn oracle_config_from_world_custom_tables() {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut lore = Entity::new(EntityKind::Lore, "Oracle Tables");
        lore.properties.insert(
            "oracle.actions".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("Attack".to_string()),
                MetadataValue::String("Defend".to_string()),
            ]),
        );
        lore.properties.insert(
            "oracle.subjects".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("Dragon".to_string()),
                MetadataValue::String("Castle".to_string()),
                MetadataValue::String("Sword".to_string()),
            ]),
        );
        world.add_entity(lore).unwrap();

        let config = OracleConfig::from_world(&world);
        assert_eq!(config.mode, OracleMode::ActionSubject);
        assert_eq!(config.actions.len(), 2);
        assert_eq!(config.subjects.len(), 3);
        assert_eq!(config.actions[0], "Attack");
        assert_eq!(config.subjects[2], "Sword");
        assert!(config.symbols.is_empty());
    }

    #[test]
    fn oracle_config_from_world_symbols() {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut lore = Entity::new(EntityKind::Lore, "Oracle Tables");
        lore.properties.insert(
            "oracle.symbols".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("Anchor".to_string()),
                MetadataValue::String("Atom".to_string()),
                MetadataValue::String("Bird".to_string()),
            ]),
        );
        world.add_entity(lore).unwrap();

        let config = OracleConfig::from_world(&world);
        assert_eq!(config.mode, OracleMode::Symbols);
        assert_eq!(config.symbols.len(), 3);
        assert_eq!(config.symbols[0], "Anchor");
        assert_eq!(config.symbols[2], "Bird");
        assert!(config.actions.is_empty());
        assert!(config.subjects.is_empty());
    }

    #[test]
    fn oracle_config_from_world_fallback_to_defaults() {
        let world = World::new(WorldMeta::new("Empty"));
        let config = OracleConfig::from_world(&world);
        assert_eq!(config.mode, OracleMode::ActionSubject);
        assert_eq!(config.actions.len(), 100);
        assert_eq!(config.subjects.len(), 100);
    }

    #[test]
    fn oracle_config_random_picks() {
        let config = OracleConfig {
            mode: OracleMode::ActionSubject,
            actions: vec!["Alpha".to_string(), "Beta".to_string()],
            subjects: vec!["One".to_string(), "Two".to_string()],
            symbols: Vec::new(),
        };
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let a = config.random_action(&mut rng);
            let s = config.random_subject(&mut rng);
            assert!(a == "Alpha" || a == "Beta");
            assert!(s == "One" || s == "Two");
        }
    }

    #[test]
    fn oracle_config_random_symbol_picks() {
        let config = OracleConfig {
            mode: OracleMode::Symbols,
            actions: Vec::new(),
            subjects: Vec::new(),
            symbols: vec![
                "First".to_string(),
                "Second".to_string(),
                "Third".to_string(),
            ],
        };
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let sym = config.random_symbol(&mut rng);
            assert!(sym == "First" || sym == "Second" || sym == "Third");
        }
    }
}
