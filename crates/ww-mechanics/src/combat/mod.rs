//! Combat state machine and participant management.
//!
//! Tracks participants, zones (optional spatial areas), turn order,
//! and a log of combat events.

pub mod action;

pub use action::{CombatAction, CombatEvent};

use crate::error::{MechError, MechResult};
use crate::sheet::CharacterSheet;

/// A participant in combat.
#[derive(Debug, Clone)]
pub struct Participant {
    /// Display name.
    pub name: String,
    /// The participant's character sheet.
    pub sheet: CharacterSheet,
    /// Which zone the participant is in.
    pub zone_index: usize,
    /// Initiative score (higher goes first).
    pub initiative: u32,
}

/// A zone in the combat area (optional spatial subdivision).
#[derive(Debug, Clone)]
pub struct Zone {
    /// Display name of the zone.
    pub name: String,
    /// Narrative traits affecting the zone (e.g., "dark", "cramped").
    pub traits: Vec<String>,
}

impl Zone {
    /// Create a new zone with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            traits: Vec::new(),
        }
    }

    /// Create a new zone with traits.
    pub fn with_traits(name: impl Into<String>, traits: Vec<String>) -> Self {
        Self {
            name: name.into(),
            traits,
        }
    }
}

/// The state of an ongoing combat encounter.
#[derive(Debug, Clone)]
pub struct Combat {
    /// All participants in the combat.
    pub participants: Vec<Participant>,
    /// Spatial zones (empty if not using zone-based combat).
    pub zones: Vec<Zone>,
    /// Current round number (1-based).
    pub round: u32,
    /// Index into `participants` for the current turn (by initiative order).
    turn_index: usize,
    /// Sorted participant indices by initiative (descending).
    initiative_order: Vec<usize>,
    /// Log of all combat events.
    pub log: Vec<CombatEvent>,
}

impl Combat {
    /// Create a new combat encounter.
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            zones: Vec::new(),
            round: 0,
            turn_index: 0,
            initiative_order: Vec::new(),
            log: Vec::new(),
        }
    }

    /// Add a participant to the combat.
    pub fn add_participant(
        &mut self,
        name: impl Into<String>,
        sheet: CharacterSheet,
        initiative: u32,
    ) {
        self.participants.push(Participant {
            name: name.into(),
            sheet,
            zone_index: 0,
            initiative,
        });
    }

    /// Add a zone to the combat area.
    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
    }

    /// Start the combat (or advance to round 1).
    /// Sorts participants by initiative and begins the first turn.
    pub fn start(&mut self) {
        self.round = 1;
        self.turn_index = 0;
        self.sort_initiative();
    }

    /// Get the index of the current participant in the initiative order.
    pub fn current_participant_index(&self) -> MechResult<usize> {
        if self.initiative_order.is_empty() {
            return Err(MechError::NoActiveParticipant);
        }
        Ok(self.initiative_order[self.turn_index])
    }

    /// Get a reference to the current participant.
    pub fn current_participant(&self) -> MechResult<&Participant> {
        let idx = self.current_participant_index()?;
        Ok(&self.participants[idx])
    }

    /// Get a mutable reference to a participant by index.
    pub fn participant_mut(&mut self, index: usize) -> MechResult<&mut Participant> {
        self.participants
            .get_mut(index)
            .ok_or(MechError::CombatError(format!(
                "participant index {index} out of bounds"
            )))
    }

    /// Advance to the next turn. Returns true if a new round started.
    pub fn next_turn(&mut self) -> bool {
        if self.initiative_order.is_empty() {
            return false;
        }
        self.turn_index += 1;
        if self.turn_index >= self.initiative_order.len() {
            self.turn_index = 0;
            self.round += 1;
            true
        } else {
            false
        }
    }

    /// Get the current round number.
    pub fn current_round(&self) -> u32 {
        self.round
    }

    /// Get the number of participants.
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Returns true if combat hasn't started yet.
    pub fn is_not_started(&self) -> bool {
        self.round == 0
    }

    /// Record a combat event in the log.
    pub fn log_event(&mut self, event: CombatEvent) {
        self.log.push(event);
    }

    /// Sort participants by initiative (descending).
    fn sort_initiative(&mut self) {
        let mut indices: Vec<usize> = (0..self.participants.len()).collect();
        indices.sort_by(|&a, &b| {
            self.participants[b]
                .initiative
                .cmp(&self.participants[a].initiative)
        });
        self.initiative_order = indices;
    }
}

impl Default for Combat {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::preset;
    use crate::sheet::CharacterSheet;
    use ww_core::entity::{Entity, EntityKind};

    fn make_sheet(name: &str) -> CharacterSheet {
        let ruleset = preset::two_d20();
        let entity = Entity::new(EntityKind::Character, name);
        CharacterSheet::from_entity(&entity, &ruleset).unwrap()
    }

    #[test]
    fn combat_lifecycle() {
        let mut combat = Combat::new();
        assert!(combat.is_not_started());
        assert_eq!(combat.participant_count(), 0);

        combat.add_participant("Alice", make_sheet("Alice"), 15);
        combat.add_participant("Bob", make_sheet("Bob"), 10);
        combat.add_participant("Charlie", make_sheet("Charlie"), 20);

        combat.start();
        assert!(!combat.is_not_started());
        assert_eq!(combat.current_round(), 1);

        // Charlie has highest initiative (20)
        let current = combat.current_participant().unwrap();
        assert_eq!(current.name, "Charlie");

        // Advance to Alice (15)
        assert!(!combat.next_turn());
        let current = combat.current_participant().unwrap();
        assert_eq!(current.name, "Alice");

        // Advance to Bob (10)
        assert!(!combat.next_turn());
        let current = combat.current_participant().unwrap();
        assert_eq!(current.name, "Bob");

        // Advance wraps to round 2
        assert!(combat.next_turn());
        assert_eq!(combat.current_round(), 2);
        let current = combat.current_participant().unwrap();
        assert_eq!(current.name, "Charlie");
    }

    #[test]
    fn combat_zones() {
        let mut combat = Combat::new();
        combat.add_zone(Zone::new("Courtyard"));
        combat.add_zone(Zone::with_traits("Tower", vec!["elevated".to_string()]));
        assert_eq!(combat.zones.len(), 2);
        assert_eq!(combat.zones[1].traits[0], "elevated");
    }

    #[test]
    fn empty_combat_error() {
        let combat = Combat::new();
        assert!(combat.current_participant().is_err());
    }

    #[test]
    fn participant_mut_access() {
        let mut combat = Combat::new();
        combat.add_participant("Alice", make_sheet("Alice"), 10);
        let p = combat.participant_mut(0).unwrap();
        p.zone_index = 1;
        assert_eq!(combat.participants[0].zone_index, 1);
    }

    #[test]
    fn participant_mut_out_of_bounds() {
        let mut combat = Combat::new();
        assert!(combat.participant_mut(0).is_err());
    }
}
