//! Choice engine for branching narratives.
//!
//! This module provides dialogue trees, conditions, and effects.

mod condition;
mod dialogue;
mod effect;
mod state;

pub use condition::Condition;
pub use dialogue::{Choice, Dialogue};
pub use effect::Effect;
pub use state::ChoiceState;
