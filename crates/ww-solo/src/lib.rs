//! Solo TTRPG runner with Mythic GME-inspired oracle.
//!
//! Provides an oracle system (fate chart for yes/no questions), random event
//! generation, NPC reaction rolls, scene management with chaos factor,
//! thread/NPC tracking, and a journaling system.

pub mod chaos;
pub mod config;
pub mod error;
pub mod journal;
pub mod oracle;
pub mod scene;
pub mod session;
pub mod tracker;

pub use chaos::ChaosFactor;
pub use config::SoloConfig;
pub use error::{SoloError, SoloResult};
pub use session::SoloSession;
