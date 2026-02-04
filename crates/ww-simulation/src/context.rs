use rand::rngs::StdRng;
use ww_core::world::World;

use crate::clock::SimClock;
use crate::event::{EventLog, SimEvent, SimEventKind};

/// Mutable context passed to each system during a tick.
pub struct SimContext<'a> {
    /// Mutable reference to the simulation world.
    pub world: &'a mut World,
    /// Reference to the simulation clock.
    pub clock: &'a SimClock,
    /// Mutable reference to the event log for recording events.
    pub events: &'a mut EventLog,
    /// Mutable reference to the deterministic random number generator.
    pub rng: &'a mut StdRng,
}

impl SimContext<'_> {
    /// Emit a simulation event at the current tick.
    pub fn emit(&mut self, kind: SimEventKind, description: impl Into<String>) {
        self.events
            .push(SimEvent::new(self.clock.tick(), kind, description));
    }

    /// Return the current simulation tick number.
    pub fn tick(&self) -> u64 {
        self.clock.tick()
    }

    /// Return the current in-world hour of day (0.0..24.0).
    pub fn hour_of_day(&self) -> f64 {
        self.clock.hour_of_day()
    }
}
