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
    /// Set the RNG seed for deterministic simulation.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set the number of in-world hours per simulation tick.
    pub fn with_hours_per_tick(mut self, hours: f64) -> Self {
        self.hours_per_tick = hours;
        self
    }

    /// Set the in-world date when the simulation begins.
    pub fn with_start_date(mut self, date: WorldDate) -> Self {
        self.start_date = date;
        self
    }

    /// Set the maximum event log size (0 = unlimited).
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_values() {
        let config = SimConfig::default();
        assert_eq!(config.seed, 42);
        assert!((config.hours_per_tick - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.max_events, 0);
        assert_eq!(config.start_date.year, 1);
    }

    #[test]
    fn config_builder_chain() {
        let config = SimConfig::default()
            .with_seed(123)
            .with_hours_per_tick(2.0)
            .with_max_events(500);
        assert_eq!(config.seed, 123);
        assert!((config.hours_per_tick - 2.0).abs() < f64::EPSILON);
        assert_eq!(config.max_events, 500);
    }

    #[test]
    fn config_start_date_builder() {
        let date = WorldDate {
            year: 5,
            month: Some(3),
            day: Some(15),
            era: None,
        };
        let config = SimConfig::default().with_start_date(date);
        assert_eq!(config.start_date.year, 5);
        assert_eq!(config.start_date.month, Some(3));
        assert_eq!(config.start_date.day, Some(15));
    }
}
