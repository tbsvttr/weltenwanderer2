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
///         scene_header "=== Log #{n} ==="
///         scene_normal "The section stretches ahead."
///         scene_altered "Something is wrong."
///         scene_interrupted "A tremor runs through the walls."
///         scene_end "Log #{n} sealed."
///         chaos_label "Pressure"
///         event_prefix "The tunnel shifts:"
///         reaction_prefix "Response"
///         enable_chaos true
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SoloWorldConfig {
    /// Custom intro/welcome text shown when the session starts.
    pub intro: Option<String>,
    /// Flavor text prefixed to oracle answers for atmosphere.
    pub oracle_prefix: Option<String>,
    /// Custom help text shown for the generic `help` command.
    pub help: Option<String>,
    /// Scene header format. Supports `{n}` placeholder for scene number.
    pub scene_header: Option<String>,
    /// Text shown when a scene proceeds normally.
    pub scene_normal: Option<String>,
    /// Text shown when a scene is altered.
    pub scene_altered: Option<String>,
    /// Text shown when a scene is interrupted.
    pub scene_interrupted: Option<String>,
    /// Scene end format. Supports `{n}` placeholder for scene number.
    pub scene_end: Option<String>,
    /// Label for the chaos factor (e.g. "Pressure", "Dread").
    pub chaos_label: Option<String>,
    /// Prefix for random event output.
    pub event_prefix: Option<String>,
    /// Prefix for NPC reaction output.
    pub reaction_prefix: Option<String>,
    /// Enable Mythic-style chaos/scene management (default: true).
    pub enable_chaos: bool,
}

impl Default for SoloWorldConfig {
    fn default() -> Self {
        Self {
            intro: None,
            oracle_prefix: None,
            help: None,
            scene_header: None,
            scene_normal: None,
            scene_altered: None,
            scene_interrupted: None,
            scene_end: None,
            chaos_label: None,
            event_prefix: None,
            reaction_prefix: None,
            enable_chaos: true, // Default to enabled for backwards compatibility
        }
    }
}

impl SoloWorldConfig {
    /// Load solo world configuration from world meta properties.
    ///
    /// Reads `solo.*` keys from the properties map.
    /// Missing keys result in `None` (fallback to defaults).
    pub fn from_world_meta(properties: &HashMap<String, MetadataValue>) -> Self {
        Self {
            intro: extract_string(properties, "solo.intro"),
            oracle_prefix: extract_string(properties, "solo.oracle_prefix"),
            help: extract_string(properties, "solo.help"),
            scene_header: extract_string(properties, "solo.scene_header"),
            scene_normal: extract_string(properties, "solo.scene_normal"),
            scene_altered: extract_string(properties, "solo.scene_altered"),
            scene_interrupted: extract_string(properties, "solo.scene_interrupted"),
            scene_end: extract_string(properties, "solo.scene_end"),
            chaos_label: extract_string(properties, "solo.chaos_label"),
            event_prefix: extract_string(properties, "solo.event_prefix"),
            reaction_prefix: extract_string(properties, "solo.reaction_prefix"),
            enable_chaos: extract_bool(properties, "solo.enable_chaos").unwrap_or(true),
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

/// Extract an optional boolean value from a properties map.
fn extract_bool(properties: &HashMap<String, MetadataValue>, key: &str) -> Option<bool> {
    match properties.get(key) {
        Some(MetadataValue::Boolean(b)) => Some(*b),
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
        assert!(cfg.enable_chaos, "chaos should be enabled by default");
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

    #[test]
    fn solo_world_config_all_new_fields() {
        let mut props = HashMap::new();
        props.insert(
            "solo.scene_header".to_string(),
            MetadataValue::String("=== Log #{n} ===".to_string()),
        );
        props.insert(
            "solo.scene_normal".to_string(),
            MetadataValue::String("All quiet.".to_string()),
        );
        props.insert(
            "solo.scene_altered".to_string(),
            MetadataValue::String("Something shifted.".to_string()),
        );
        props.insert(
            "solo.scene_interrupted".to_string(),
            MetadataValue::String("A tremor!".to_string()),
        );
        props.insert(
            "solo.scene_end".to_string(),
            MetadataValue::String("Log #{n} sealed.".to_string()),
        );
        props.insert(
            "solo.chaos_label".to_string(),
            MetadataValue::String("Pressure".to_string()),
        );
        props.insert(
            "solo.event_prefix".to_string(),
            MetadataValue::String("The tunnel shifts:".to_string()),
        );
        props.insert(
            "solo.reaction_prefix".to_string(),
            MetadataValue::String("Response".to_string()),
        );

        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert_eq!(cfg.scene_header.as_deref(), Some("=== Log #{n} ==="));
        assert_eq!(cfg.scene_normal.as_deref(), Some("All quiet."));
        assert_eq!(cfg.scene_altered.as_deref(), Some("Something shifted."));
        assert_eq!(cfg.scene_interrupted.as_deref(), Some("A tremor!"));
        assert_eq!(cfg.scene_end.as_deref(), Some("Log #{n} sealed."));
        assert_eq!(cfg.chaos_label.as_deref(), Some("Pressure"));
        assert_eq!(cfg.event_prefix.as_deref(), Some("The tunnel shifts:"));
        assert_eq!(cfg.reaction_prefix.as_deref(), Some("Response"));
    }

    #[test]
    fn solo_world_config_new_fields_default_none() {
        let props = HashMap::new();
        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert!(cfg.scene_header.is_none());
        assert!(cfg.scene_normal.is_none());
        assert!(cfg.scene_altered.is_none());
        assert!(cfg.scene_interrupted.is_none());
        assert!(cfg.scene_end.is_none());
        assert!(cfg.chaos_label.is_none());
        assert!(cfg.event_prefix.is_none());
        assert!(cfg.reaction_prefix.is_none());
    }

    #[test]
    fn solo_world_config_chaos_can_be_disabled() {
        let mut props = HashMap::new();
        props.insert(
            "solo.enable_chaos".to_string(),
            MetadataValue::Boolean(false),
        );
        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert!(
            !cfg.enable_chaos,
            "chaos should be disabled when set to false"
        );
    }

    #[test]
    fn solo_world_config_chaos_enabled_explicitly() {
        let mut props = HashMap::new();
        props.insert(
            "solo.enable_chaos".to_string(),
            MetadataValue::Boolean(true),
        );
        let cfg = SoloWorldConfig::from_world_meta(&props);
        assert!(cfg.enable_chaos, "chaos should be enabled when set to true");
    }
}
