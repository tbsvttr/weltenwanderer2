//! Tick-based world simulation for Weltenwanderer.
//!
//! Provides a system-based simulation framework operating on a [`ww_core::World`].
//! Simulation state (needs, schedules, spatial tracking) is stored externally,
//! keeping ww-core clean. The simulation reads from and optionally writes to
//! entity components when appropriate (e.g., updating character status on death).

/// Simulation clock for tracking ticks and in-world time.
pub mod clock;
/// Configuration types for simulation runs.
pub mod config;
/// Mutable context passed to systems each tick.
pub mod context;
/// Error types for the simulation crate.
pub mod error;
/// Simulation event types and the event log.
pub mod event;
/// Needs system: tracks and decays entity needs like hunger and rest.
pub mod needs;
/// Schedule system: assigns time-based activities to entities.
pub mod schedule;
/// Top-level simulation orchestrator.
pub mod simulation;
/// Spatial system: tracks entity locations and movement.
pub mod spatial;
/// The trait that all simulation systems implement.
pub mod system;

/// Re-export of [`clock::SimClock`].
pub use clock::SimClock;
/// Re-export of [`config::SimConfig`].
pub use config::SimConfig;
/// Re-export of [`context::SimContext`].
pub use context::SimContext;
/// Re-exports of [`error::SimError`] and [`error::SimResult`].
pub use error::{SimError, SimResult};
/// Re-exports of [`event::EventLog`], [`event::SimEvent`], and [`event::SimEventKind`].
pub use event::{EventLog, SimEvent, SimEventKind};
/// Re-export of [`simulation::Simulation`].
pub use simulation::Simulation;
/// Re-export of [`system::System`].
pub use system::System;
