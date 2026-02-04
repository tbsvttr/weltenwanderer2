use ww_core::component::WorldDate;

/// Configuration for a simulation run.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// RNG seed for deterministic simulation.
    pub seed: u64,
    /// In-world hours per simulation tick.
    pub hours_per_tick: f64,
    /// The in-world date when the simulation begins.
    pub start_date: WorldDate,
    /// Maximum event log size (oldest events dropped when exceeded). 0 = unlimited.
    pub max_events: usize,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            hours_per_tick: 1.0,
            start_date: WorldDate::new(1),
            max_events: 0,
        }
    }
}

impl SimConfig {
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_hours_per_tick(mut self, hours: f64) -> Self {
        self.hours_per_tick = hours;
        self
    }

    pub fn with_start_date(mut self, date: WorldDate) -> Self {
        self.start_date = date;
        self
    }

    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }
}
