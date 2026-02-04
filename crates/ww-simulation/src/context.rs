use rand::rngs::StdRng;
use ww_core::world::World;

use crate::clock::SimClock;
use crate::event::{EventLog, SimEvent, SimEventKind};

/// Mutable context passed to each system during a tick.
pub struct SimContext<'a> {
    pub world: &'a mut World,
    pub clock: &'a SimClock,
    pub events: &'a mut EventLog,
    pub rng: &'a mut StdRng,
}

impl SimContext<'_> {
    /// Emit a simulation event at the current tick.
    pub fn emit(&mut self, kind: SimEventKind, description: impl Into<String>) {
        self.events
            .push(SimEvent::new(self.clock.tick(), kind, description));
    }

    pub fn tick(&self) -> u64 {
        self.clock.tick()
    }

    pub fn hour_of_day(&self) -> f64 {
        self.clock.hour_of_day()
    }
}
