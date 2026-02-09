//! Configuration for a solo TTRPG session.

use std::collections::HashMap;

use ww_core::entity::MetadataValue;

/// Configuration for a solo session.
#[derive(Debug, Clone)]
pub struct SoloConfig {
    /// RNG seed for reproducible oracle rolls.
    pub seed: u64,
    /// Initial chaos factor (1-9).
    pub initial_chaos: u32,
}

impl Default for SoloConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            initial_chaos: 5,
        }
    }
}

impl SoloConfig {
    /// Set the RNG seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set the initial chaos factor (clamped to 1-9).
    pub fn with_chaos(mut self, chaos: u32) -> Self {
        self.initial_chaos = chaos.clamp(1, 9);
        self
    }
}

/// World-level solo session configuration, loaded from the `solo { }` DSL block.
///
/// World authors can customize the solo experience by adding a `solo { }` block
/// inside their `world { }` declaration. Properties are stored as `solo.*` keys
/// in `WorldMeta.properties` via the compiler's generic block flattening.
///
/// # Example DSL
///
/// ```text
/// world "My World" {
///     solo {
///         intro "Welcome to the adventure..."
///         oracle_prefix "The spirits whisper..."
///         help "Use 'go' to move, 'ask' for the oracle."
///     }
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct SoloWorldConfig {
    /// Custom intro/welcome text shown when the session starts.
    pub intro: Option<String>,
    /// Flavor text prefixed to oracle answers for atmosphere.
    pub oracle_prefix: Option<String>,
    /// Custom help text shown for the generic `help` command.
    pub help: Option<String>,
}

impl SoloWorldConfig {
    /// Load solo world configuration from world meta properties.
    ///
    /// Reads `solo.intro`, `solo.oracle_prefix`, and `solo.help` from the
    /// properties map. Missing keys result in `None` (fallback to defaults).
    pub fn from_world_meta(properties: &HashMap<String, MetadataValue>) -> Self {
        Self {
            intro: extract_string(properties, "solo.intro"),
            oracle_prefix: extract_string(properties, "solo.oracle_prefix"),
            help: extract_string(properties, "solo.help"),
        }
    }
}

/// Extract an optional string value from a properties map.
fn extract_string(properties: &HashMap<String, MetadataValue>, key: &str) -> Option<String> {
    match properties.get(key) {
        Some(MetadataValue::String(s)) => Some(s.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = SoloConfig::default();
        assert_eq!(cfg.seed, 42);
        assert_eq!(cfg.initial_chaos, 5);
    }

    #[test]
    fn builder_methods() {
        let cfg = SoloConfig::default().with_seed(123).with_chaos(8);
        assert_eq!(cfg.seed, 123);
        assert_eq!(cfg.initial_chaos, 8);
    }

    #[test]
    fn chaos_clamped() {
        let cfg = SoloConfig::default().with_chaos(0);
        assert_eq!(cfg.initial_chaos, 1);
        let cfg = SoloConfig::default().with_chaos(99);
        assert_eq!(cfg.initial_chaos, 9);
    }

    #[test]
    fn solo_world_config_from_empty() {
        let props = HashMap::new();
        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert!(cfg.intro.is_none());
        assert!(cfg.oracle_prefix.is_none());
        assert!(cfg.help.is_none());
    }

    #[test]
    fn solo_world_config_from_properties() {
        let mut props = HashMap::new();
        props.insert(
            "solo.intro".to_string(),
            MetadataValue::String("Welcome!".to_string()),
        );
        props.insert(
            "solo.oracle_prefix".to_string(),
            MetadataValue::String("The spirits say...".to_string()),
        );
        props.insert(
            "solo.help".to_string(),
            MetadataValue::String("Type 'go' to move.".to_string()),
        );
        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert_eq!(cfg.intro.as_deref(), Some("Welcome!"));
        assert_eq!(cfg.oracle_prefix.as_deref(), Some("The spirits say..."));
        assert_eq!(cfg.help.as_deref(), Some("Type 'go' to move."));
    }

    #[test]
    fn solo_world_config_ignores_non_string() {
        let mut props = HashMap::new();
        props.insert("solo.intro".to_string(), MetadataValue::Integer(42));
        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert!(cfg.intro.is_none());
    }
}
