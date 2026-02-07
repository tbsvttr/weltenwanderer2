//! Interactive fiction engine for Weltenwanderer.
//!
//! Provides a text-adventure-style interface for exploring worlds built with
//! the Weltenwanderer DSL. Features include natural language command parsing,
//! fuzzy entity name resolution, branching dialogue trees, a narrator system
//! with configurable tone, and simulation integration.

/// Choice engine for branching narratives.
pub mod choice;
/// Error types for the fiction engine.
pub mod error;
/// Narrator system for descriptive text generation.
pub mod narrator;
/// Command parsing and entity resolution.
pub mod parser;
/// Player state management.
pub mod player;
/// Interactive fiction session management.
pub mod session;
/// Fiction system for simulation integration.
pub mod system;

pub use error::{FictionError, FictionResult};
pub use parser::{Command, Direction, parse_command};
pub use player::PlayerState;
pub use session::FictionSession;
pub use system::FictionSystem;
