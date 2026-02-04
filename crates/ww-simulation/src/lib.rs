//! Tick-based world simulation for Weltenwanderer.
//!
//! Provides a system-based simulation framework operating on a [`ww_core::World`].
//! Simulation state (needs, schedules, spatial tracking) is stored externally,
//! keeping ww-core clean. The simulation reads from and optionally writes to
//! entity components when appropriate (e.g., updating character status on death).

pub mod clock;
pub mod config;
pub mod context;
pub mod error;
pub mod event;
pub mod needs;
pub mod schedule;
pub mod simulation;
pub mod spatial;
pub mod system;

pub use clock::SimClock;
pub use config::SimConfig;
pub use context::SimContext;
pub use error::{SimError, SimResult};
pub use event::{EventLog, SimEvent, SimEventKind};
pub use simulation::Simulation;
pub use system::System;
