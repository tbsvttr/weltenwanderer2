//! Combat actions and event logging.

use rand::rngs::StdRng;

use crate::error::{MechError, MechResult};
use crate::rules::{self, CheckRequest, CheckResult, RuleSet};
use crate::sheet::CharacterSheet;

use super::Combat;

/// An action a participant can take during their turn.
#[derive(Debug, Clone)]
pub enum CombatAction {
    /// Attack a target participant.
    Attack {
        /// Index of the target participant.
        target: usize,
    },
    /// Take a defensive stance.
    Defend,
    /// Move to a different zone.
    Move {
        /// Index of the destination zone.
        to_zone: usize,
    },
    /// Use a skill, optionally targeting a participant.
    UseSkill {
        /// Name of the skill to use.
        skill: String,
        /// Optional target participant index.
        target: Option<usize>,
    },
    /// A free-form action described by text.
    Custom(String),
}

impl std::fmt::Display for CombatAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attack { target } => write!(f, "Attack target #{target}"),
            Self::Defend => write!(f, "Defend"),
            Self::Move { to_zone } => write!(f, "Move to zone #{to_zone}"),
            Self::UseSkill { skill, target } => {
                if let Some(t) = target {
                    write!(f, "Use {skill} on #{t}")
                } else {
                    write!(f, "Use {skill}")
                }
            }
            Self::Custom(desc) => write!(f, "{desc}"),
        }
    }
}

/// A recorded combat event.
#[derive(Debug, Clone)]
pub struct CombatEvent {
    /// Which round this happened in.
    pub round: u32,
    /// Index of the acting participant.
    pub actor: usize,
    /// What action was taken.
    pub action: CombatAction,
    /// The mechanical result of the action.
    pub result: CheckResult,
    /// A narrative description of what happened.
    pub description: String,
}

/// Resolve a combat action, producing a check result and effects.
pub fn resolve_action(
    combat: &Combat,
    ruleset: &RuleSet,
    actor_index: usize,
    action: &CombatAction,
    rng: &mut StdRng,
) -> MechResult<CheckResult> {
    let actor = combat.participants.get(actor_index).ok_or_else(|| {
        MechError::CombatError(format!("actor index {actor_index} out of bounds"))
    })?;

    let request = build_request_for_action(action, &actor.sheet, ruleset)?;
    rules::perform_check(ruleset, &actor.sheet, &request, rng)
}

/// Build a check request appropriate for the given action.
fn build_request_for_action(
    action: &CombatAction,
    sheet: &CharacterSheet,
    ruleset: &RuleSet,
) -> MechResult<CheckRequest> {
    match action {
        CombatAction::Attack { .. } => {
            // Use the first combat-relevant attribute and skill
            let attribute = find_combat_attribute(ruleset);
            let skill = find_combat_skill(ruleset);
            Ok(CheckRequest {
                attribute: Some(attribute),
                skill: Some(skill),
                difficulty: Some(1),
                ..CheckRequest::default()
            })
        }
        CombatAction::Defend => {
            let attribute = find_defense_attribute(ruleset);
            Ok(CheckRequest {
                attribute: Some(attribute),
                difficulty: Some(1),
                ..CheckRequest::default()
            })
        }
        CombatAction::UseSkill { skill, .. } => {
            // Validate skill exists
            let _ = sheet.skill(skill);
            Ok(CheckRequest {
                skill: Some(skill.clone()),
                difficulty: Some(1),
                ..CheckRequest::default()
            })
        }
        CombatAction::Move { to_zone } => {
            // Movement is usually free but may require a check
            let _ = to_zone;
            Ok(CheckRequest::default())
        }
        CombatAction::Custom(_) => Ok(CheckRequest::default()),
    }
}

/// Find the best attribute for attack checks in this ruleset.
fn find_combat_attribute(ruleset: &RuleSet) -> String {
    let combat_attrs = ["Brawn", "Prowess", "Strength", "Agility"];
    for attr in &combat_attrs {
        if ruleset
            .attributes
            .iter()
            .any(|a| a.eq_ignore_ascii_case(attr))
        {
            return (*attr).to_string();
        }
    }
    ruleset
        .attributes
        .first()
        .cloned()
        .unwrap_or_else(|| "Brawn".to_string())
}

/// Find the best attribute for defense checks in this ruleset.
fn find_defense_attribute(ruleset: &RuleSet) -> String {
    let defense_attrs = ["Agility", "Finesse", "Coordination", "Cunning"];
    for attr in &defense_attrs {
        if ruleset
            .attributes
            .iter()
            .any(|a| a.eq_ignore_ascii_case(attr))
        {
            return (*attr).to_string();
        }
    }
    ruleset
        .attributes
        .first()
        .cloned()
        .unwrap_or_else(|| "Agility".to_string())
}

/// Find the best skill for combat checks in this ruleset.
fn find_combat_skill(ruleset: &RuleSet) -> String {
    let combat_skills = ["Melee", "Fighting", "Warfare", "Athletics"];
    for skill in &combat_skills {
        if ruleset.skills.iter().any(|s| s.eq_ignore_ascii_case(skill)) {
            return (*skill).to_string();
        }
    }
    ruleset
        .skills
        .first()
        .cloned()
        .unwrap_or_else(|| "Melee".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{Combat, Zone};
    use crate::rules::preset;
    use rand::SeedableRng;
    use ww_core::entity::{Entity, EntityKind, MetadataValue};

    fn setup_combat() -> (Combat, RuleSet) {
        let ruleset = preset::two_d20();
        let mut combat = Combat::new();

        let mut entity1 = Entity::new(EntityKind::Character, "Alice");
        entity1
            .properties
            .insert("mechanics.agility".to_string(), MetadataValue::Integer(12));
        entity1
            .properties
            .insert("mechanics.brawn".to_string(), MetadataValue::Integer(10));
        entity1
            .properties
            .insert("mechanics.melee".to_string(), MetadataValue::Integer(3));

        let mut entity2 = Entity::new(EntityKind::Character, "Bob");
        entity2
            .properties
            .insert("mechanics.agility".to_string(), MetadataValue::Integer(8));
        entity2
            .properties
            .insert("mechanics.brawn".to_string(), MetadataValue::Integer(14));

        let sheet1 = CharacterSheet::from_entity(&entity1, &ruleset).unwrap();
        let sheet2 = CharacterSheet::from_entity(&entity2, &ruleset).unwrap();

        combat.add_participant("Alice", sheet1, 15);
        combat.add_participant("Bob", sheet2, 10);
        combat.add_zone(Zone::new("Arena"));
        combat.start();

        (combat, ruleset)
    }

    #[test]
    fn resolve_attack() {
        let (combat, ruleset) = setup_combat();
        let mut rng = StdRng::seed_from_u64(42);
        let result = resolve_action(
            &combat,
            &ruleset,
            0,
            &CombatAction::Attack { target: 1 },
            &mut rng,
        )
        .unwrap();
        assert!(!result.roll.dice.is_empty());
    }

    #[test]
    fn resolve_defend() {
        let (combat, ruleset) = setup_combat();
        let mut rng = StdRng::seed_from_u64(42);
        let result = resolve_action(&combat, &ruleset, 1, &CombatAction::Defend, &mut rng).unwrap();
        assert!(!result.roll.dice.is_empty());
    }

    #[test]
    fn resolve_use_skill() {
        let (combat, ruleset) = setup_combat();
        let mut rng = StdRng::seed_from_u64(42);
        let result = resolve_action(
            &combat,
            &ruleset,
            0,
            &CombatAction::UseSkill {
                skill: "Melee".to_string(),
                target: Some(1),
            },
            &mut rng,
        )
        .unwrap();
        assert!(!result.roll.dice.is_empty());
    }

    #[test]
    fn resolve_invalid_actor() {
        let (combat, ruleset) = setup_combat();
        let mut rng = StdRng::seed_from_u64(42);
        assert!(resolve_action(&combat, &ruleset, 99, &CombatAction::Defend, &mut rng).is_err());
    }

    #[test]
    fn combat_action_display() {
        assert_eq!(
            CombatAction::Attack { target: 1 }.to_string(),
            "Attack target #1"
        );
        assert_eq!(CombatAction::Defend.to_string(), "Defend");
        assert_eq!(
            CombatAction::Move { to_zone: 2 }.to_string(),
            "Move to zone #2"
        );
        assert_eq!(
            CombatAction::UseSkill {
                skill: "Stealth".to_string(),
                target: None
            }
            .to_string(),
            "Use Stealth"
        );
        assert_eq!(
            CombatAction::Custom("Rally allies".to_string()).to_string(),
            "Rally allies"
        );
    }

    #[test]
    fn find_combat_attrs_2d20() {
        let ruleset = preset::two_d20();
        assert_eq!(find_combat_attribute(&ruleset), "Brawn");
        assert_eq!(find_defense_attribute(&ruleset), "Agility");
        assert_eq!(find_combat_skill(&ruleset), "Melee");
    }

    #[test]
    fn find_combat_attrs_blood_honor() {
        let ruleset = preset::blood_and_honor();
        assert_eq!(find_combat_attribute(&ruleset), "Prowess");
        assert_eq!(find_defense_attribute(&ruleset), "Cunning");
        assert_eq!(find_combat_skill(&ruleset), "Warfare");
    }

    #[test]
    fn find_combat_attrs_trophy_gold() {
        let ruleset = preset::trophy_gold();
        assert_eq!(find_combat_attribute(&ruleset), "Brawn");
        assert_eq!(find_defense_attribute(&ruleset), "Finesse");
        assert_eq!(find_combat_skill(&ruleset), "Fighting");
    }
}
