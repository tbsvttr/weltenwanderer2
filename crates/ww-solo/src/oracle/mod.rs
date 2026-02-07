//! Oracle system for solo TTRPG play.
//!
//! Provides a fate chart for yes/no questions, random event generation,
//! NPC reaction rolls, and meaning tables for event interpretation.

pub mod event;
pub mod fate_chart;
pub mod reaction;
pub mod tables;

pub use event::{EventFocus, RandomEvent, generate_random_event};
pub use fate_chart::{Likelihood, OracleAnswer, OracleResult, consult_oracle};
pub use reaction::{NpcReaction, roll_npc_reaction};
pub use tables::OracleConfig;
