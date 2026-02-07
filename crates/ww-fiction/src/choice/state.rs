//! State tracking for dialogue choices.

use std::collections::{HashMap, HashSet};

/// Tracks which dialogues and choices the player has seen.
#[derive(Debug, Clone, Default)]
pub struct ChoiceState {
    /// Dialogues that have been started.
    seen_dialogues: HashSet<String>,
    /// Choices that have been selected (dialogue_id -> choice indices).
    selected_choices: HashMap<String, HashSet<usize>>,
    /// Dialogues that have been completed (all choices exhausted).
    completed_dialogues: HashSet<String>,
}

impl ChoiceState {
    /// Create a new empty choice state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a dialogue as seen.
    pub fn mark_seen(&mut self, dialogue_id: impl Into<String>) {
        self.seen_dialogues.insert(dialogue_id.into());
    }

    /// Check if a dialogue has been seen.
    pub fn has_seen(&self, dialogue_id: &str) -> bool {
        self.seen_dialogues.contains(dialogue_id)
    }

    /// Mark a choice as selected.
    pub fn mark_choice(&mut self, dialogue_id: impl Into<String>, choice_index: usize) {
        self.selected_choices
            .entry(dialogue_id.into())
            .or_default()
            .insert(choice_index);
    }

    /// Check if a specific choice has been selected.
    pub fn has_selected_choice(&self, dialogue_id: &str, choice_index: usize) -> bool {
        self.selected_choices
            .get(dialogue_id)
            .is_some_and(|choices| choices.contains(&choice_index))
    }

    /// Get all selected choices for a dialogue.
    pub fn selected_choices_for(&self, dialogue_id: &str) -> Option<&HashSet<usize>> {
        self.selected_choices.get(dialogue_id)
    }

    /// Mark a dialogue as completed.
    pub fn mark_completed(&mut self, dialogue_id: impl Into<String>) {
        self.completed_dialogues.insert(dialogue_id.into());
    }

    /// Check if a dialogue is completed.
    pub fn is_completed(&self, dialogue_id: &str) -> bool {
        self.completed_dialogues.contains(dialogue_id)
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.seen_dialogues.clear();
        self.selected_choices.clear();
        self.completed_dialogues.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_seen_dialogues() {
        let mut state = ChoiceState::new();
        assert!(!state.has_seen("greeting"));

        state.mark_seen("greeting");
        assert!(state.has_seen("greeting"));
    }

    #[test]
    fn track_selected_choices() {
        let mut state = ChoiceState::new();
        assert!(!state.has_selected_choice("greeting", 0));

        state.mark_choice("greeting", 0);
        assert!(state.has_selected_choice("greeting", 0));
        assert!(!state.has_selected_choice("greeting", 1));
    }

    #[test]
    fn track_completed() {
        let mut state = ChoiceState::new();
        assert!(!state.is_completed("greeting"));

        state.mark_completed("greeting");
        assert!(state.is_completed("greeting"));
    }

    #[test]
    fn reset_state() {
        let mut state = ChoiceState::new();
        state.mark_seen("greeting");
        state.mark_choice("greeting", 0);
        state.mark_completed("greeting");

        state.reset();

        assert!(!state.has_seen("greeting"));
        assert!(!state.has_selected_choice("greeting", 0));
        assert!(!state.is_completed("greeting"));
    }
}
