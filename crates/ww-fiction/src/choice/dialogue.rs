//! Dialogue trees and choice structures.

use super::condition::Condition;
use super::effect::Effect;

/// A dialogue tree for a character.
#[derive(Debug, Clone)]
pub struct Dialogue {
    /// Unique identifier for this dialogue.
    pub id: String,
    /// The character who speaks this dialogue (entity name).
    pub speaker: Option<String>,
    /// Conditions that must be met to show this dialogue.
    pub conditions: Vec<Condition>,
    /// The dialogue text to display.
    pub text: String,
    /// Available choices.
    pub choices: Vec<Choice>,
}

impl Dialogue {
    /// Create a new dialogue with the given ID and text.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            speaker: None,
            conditions: Vec::new(),
            text: text.into(),
            choices: Vec::new(),
        }
    }

    /// Set the speaker.
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Add a condition.
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add a choice.
    pub fn with_choice(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }
}

/// A single choice in a dialogue.
#[derive(Debug, Clone)]
pub struct Choice {
    /// The text shown to the player.
    pub text: String,
    /// Conditions that must be met to show this choice.
    pub conditions: Vec<Condition>,
    /// The response text shown when this choice is selected.
    pub response: String,
    /// Effects to apply when this choice is selected.
    pub effects: Vec<Effect>,
    /// Dialogue ID to branch to after this choice (if any).
    pub goto: Option<String>,
}

impl Choice {
    /// Create a new choice with the given text and response.
    pub fn new(text: impl Into<String>, response: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            conditions: Vec::new(),
            response: response.into(),
            effects: Vec::new(),
            goto: None,
        }
    }

    /// Add a condition.
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add an effect.
    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    /// Set the goto branch.
    pub fn with_goto(mut self, dialogue_id: impl Into<String>) -> Self {
        self.goto = Some(dialogue_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_builder() {
        let dialogue = Dialogue::new("greeting", "Hello, traveler!")
            .with_speaker("Innkeeper")
            .with_condition(Condition::Always)
            .with_choice(Choice::new("Hello!", "Welcome to my inn."));

        assert_eq!(dialogue.id, "greeting");
        assert_eq!(dialogue.speaker, Some("Innkeeper".to_string()));
        assert_eq!(dialogue.choices.len(), 1);
    }

    #[test]
    fn choice_builder() {
        let choice = Choice::new("Ask about rumors", "I've heard strange things...")
            .with_condition(Condition::HasKnowledge {
                key: "met_innkeeper".to_string(),
            })
            .with_effect(Effect::SetKnowledge {
                key: "heard_rumors".to_string(),
                value: true,
            })
            .with_goto("rumors_branch");

        assert_eq!(choice.text, "Ask about rumors");
        assert_eq!(choice.goto, Some("rumors_branch".to_string()));
        assert_eq!(choice.conditions.len(), 1);
        assert_eq!(choice.effects.len(), 1);
    }
}
