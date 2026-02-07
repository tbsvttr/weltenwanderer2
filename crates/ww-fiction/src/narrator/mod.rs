//! Narrator system for generating descriptive text.

mod config;
mod templates;

pub use config::{NarratorConfig, NarratorTone, Perspective, Verbosity};
pub use templates::TemplateRegistry;
