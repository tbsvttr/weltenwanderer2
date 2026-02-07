//! Narrator configuration.

/// Narrator tone - affects the style of descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NarratorTone {
    /// Formal, traditional style.
    #[default]
    Formal,
    /// Casual, conversational style.
    Casual,
    /// Dramatic, epic style.
    Dramatic,
    /// Humorous, lighthearted style.
    Humorous,
}

/// Narrative perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Perspective {
    /// Second person ("You enter the room").
    #[default]
    SecondPerson,
    /// Third person ("The hero enters the room").
    ThirdPerson,
}

/// Verbosity level for descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Minimal descriptions.
    Terse,
    /// Standard descriptions.
    #[default]
    Normal,
    /// Detailed, elaborate descriptions.
    Verbose,
}

/// Configuration for the narrator.
#[derive(Debug, Clone, Default)]
pub struct NarratorConfig {
    /// The tone of narration.
    pub tone: NarratorTone,
    /// The narrative perspective.
    pub perspective: Perspective,
    /// The verbosity level.
    pub verbosity: Verbosity,
    /// Name to use for the player in third person.
    pub player_name: Option<String>,
}

impl NarratorConfig {
    /// Create a new narrator config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tone.
    pub fn with_tone(mut self, tone: NarratorTone) -> Self {
        self.tone = tone;
        self
    }

    /// Set the perspective.
    pub fn with_perspective(mut self, perspective: Perspective) -> Self {
        self.perspective = perspective;
        self
    }

    /// Set the verbosity.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Set the player name for third person.
    pub fn with_player_name(mut self, name: impl Into<String>) -> Self {
        self.player_name = Some(name.into());
        self
    }

    /// Get the subject pronoun for the player.
    pub fn player_subject(&self) -> &str {
        match self.perspective {
            Perspective::SecondPerson => "You",
            Perspective::ThirdPerson => self.player_name.as_deref().unwrap_or("The hero"),
        }
    }

    /// Get the object pronoun for the player.
    pub fn player_object(&self) -> &str {
        match self.perspective {
            Perspective::SecondPerson => "you",
            Perspective::ThirdPerson => self.player_name.as_deref().unwrap_or("the hero"),
        }
    }

    /// Get the possessive for the player.
    pub fn player_possessive(&self) -> &str {
        match self.perspective {
            Perspective::SecondPerson => "your",
            Perspective::ThirdPerson => {
                // Could do "the hero's" but that's awkward
                self.player_name
                    .as_deref()
                    .map(|_| "their")
                    .unwrap_or("their")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = NarratorConfig::new();
        assert_eq!(config.tone, NarratorTone::Formal);
        assert_eq!(config.perspective, Perspective::SecondPerson);
        assert_eq!(config.verbosity, Verbosity::Normal);
    }

    #[test]
    fn builder_pattern() {
        let config = NarratorConfig::new()
            .with_tone(NarratorTone::Dramatic)
            .with_perspective(Perspective::ThirdPerson)
            .with_verbosity(Verbosity::Verbose)
            .with_player_name("Kael");

        assert_eq!(config.tone, NarratorTone::Dramatic);
        assert_eq!(config.perspective, Perspective::ThirdPerson);
        assert_eq!(config.player_subject(), "Kael");
    }

    #[test]
    fn pronouns_second_person() {
        let config = NarratorConfig::new();
        assert_eq!(config.player_subject(), "You");
        assert_eq!(config.player_object(), "you");
        assert_eq!(config.player_possessive(), "your");
    }

    #[test]
    fn pronouns_third_person() {
        let config = NarratorConfig::new().with_perspective(Perspective::ThirdPerson);
        assert_eq!(config.player_subject(), "The hero");

        let config = config.with_player_name("Kael");
        assert_eq!(config.player_subject(), "Kael");
    }
}
