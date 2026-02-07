//! Random event generation.
//!
//! When the oracle triggers a random event, we roll on the event focus table
//! to determine what kind of event occurs, then pick action and subject words
//! for the player to interpret narratively.

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use super::tables::OracleConfig;

/// What a random event is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventFocus {
    /// Something happens elsewhere that affects the story.
    RemoteEvent,
    /// An NPC takes independent action.
    NpcAction,
    /// A new NPC enters the story.
    IntroduceNpc,
    /// An active plot thread advances.
    MoveTowardThread,
    /// An active plot thread is set back.
    MoveAwayFromThread,
    /// An active plot thread resolves.
    CloseThread,
    /// Something bad happens to the player character.
    PcNegative,
    /// Something good happens to the player character.
    PcPositive,
    /// An ambiguous event that could go either way.
    AmbiguousEvent,
    /// Something bad happens to an NPC.
    NpcNegative,
    /// Something good happens to an NPC.
    NpcPositive,
}

impl std::fmt::Display for EventFocus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RemoteEvent => write!(f, "Remote Event"),
            Self::NpcAction => write!(f, "NPC Action"),
            Self::IntroduceNpc => write!(f, "Introduce NPC"),
            Self::MoveTowardThread => write!(f, "Move Toward Thread"),
            Self::MoveAwayFromThread => write!(f, "Move Away From Thread"),
            Self::CloseThread => write!(f, "Close Thread"),
            Self::PcNegative => write!(f, "PC Negative"),
            Self::PcPositive => write!(f, "PC Positive"),
            Self::AmbiguousEvent => write!(f, "Ambiguous Event"),
            Self::NpcNegative => write!(f, "NPC Negative"),
            Self::NpcPositive => write!(f, "NPC Positive"),
        }
    }
}

/// A generated random event with focus and meaning descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEvent {
    /// What the event is about.
    pub focus: EventFocus,
    /// Action descriptor (verb/concept).
    pub action: String,
    /// Subject descriptor (noun/concept).
    pub subject: String,
}

impl std::fmt::Display for RandomEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} + {}", self.focus, self.action, self.subject)
    }
}

/// Roll on the event focus table (d100).
pub fn roll_event_focus(rng: &mut StdRng) -> EventFocus {
    let roll: u32 = rng.random_range(1..=100);
    match roll {
        1..=7 => EventFocus::RemoteEvent,
        8..=28 => EventFocus::NpcAction,
        29..=35 => EventFocus::IntroduceNpc,
        36..=45 => EventFocus::MoveTowardThread,
        46..=52 => EventFocus::MoveAwayFromThread,
        53..=55 => EventFocus::CloseThread,
        56..=67 => EventFocus::PcNegative,
        68..=75 => EventFocus::PcPositive,
        76..=83 => EventFocus::AmbiguousEvent,
        84..=92 => EventFocus::NpcNegative,
        93..=100 => EventFocus::NpcPositive,
        _ => unreachable!(),
    }
}

/// Generate a complete random event using the given oracle configuration.
pub fn generate_random_event(rng: &mut StdRng, config: &OracleConfig) -> RandomEvent {
    let focus = roll_event_focus(rng);
    let action = config.random_action(rng).to_string();
    let subject = config.random_subject(rng).to_string();
    RandomEvent {
        focus,
        action,
        subject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn event_focus_covers_full_range() {
        // Roll every value 1-100 and ensure all are valid
        let mut seen = std::collections::HashSet::new();
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..1000 {
            let focus = roll_event_focus(&mut rng);
            seen.insert(format!("{focus:?}"));
        }
        // Should see all 11 focus types
        assert_eq!(seen.len(), 11, "missing focus types: {seen:?}");
    }

    #[test]
    fn generate_event_has_all_fields() {
        let mut rng = StdRng::seed_from_u64(42);
        let config = OracleConfig::default();
        let event = generate_random_event(&mut rng, &config);
        assert!(!event.action.is_empty());
        assert!(!event.subject.is_empty());
    }

    #[test]
    fn event_display() {
        let event = RandomEvent {
            focus: EventFocus::NpcAction,
            action: "Betray".to_string(),
            subject: "Allies".to_string(),
        };
        assert_eq!(event.to_string(), "NPC Action: Betray + Allies");
    }

    #[test]
    fn event_focus_display() {
        assert_eq!(EventFocus::RemoteEvent.to_string(), "Remote Event");
        assert_eq!(EventFocus::PcPositive.to_string(), "PC Positive");
        assert_eq!(EventFocus::CloseThread.to_string(), "Close Thread");
    }

    #[test]
    fn event_serde_roundtrip() {
        let event = RandomEvent {
            focus: EventFocus::IntroduceNpc,
            action: "Create".to_string(),
            subject: "Enemies".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let event2: RandomEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event2.focus, EventFocus::IntroduceNpc);
        assert_eq!(event2.action, "Create");
        assert_eq!(event2.subject, "Enemies");
    }
}
