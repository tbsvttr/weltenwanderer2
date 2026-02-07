//! TTRPG game mechanics engine for Weltenwanderer.
//!
//! Provides dice rolling, resolution strategies, character sheets,
//! a rules engine with DSL-configurable rulesets, and a combat system.
//! Ships with three reference systems: 2d20 (Modiphius), Trophy Gold,
//! and Blood & Honor.

pub mod combat;
pub mod dice;
pub mod error;
pub mod resolution;
pub mod rules;
pub mod sheet;
pub mod validate;

pub use dice::{DicePool, DiceTag, Die, DieResult, RollResult};
pub use error::{MechError, MechResult};
pub use resolution::{CountSuccesses, HighestDie, Outcome, ResolutionStrategy, SumPool};
pub use rules::{CheckEffect, CheckRequest, CheckResult, RuleSet, TrackDefinition};
pub use sheet::{CharacterSheet, Track};
pub use validate::validate_world;
