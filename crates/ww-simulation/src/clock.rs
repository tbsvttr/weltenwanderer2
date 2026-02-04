use ww_core::component::WorldDate;

/// Tracks simulation time: a monotonic tick counter and an in-world date.
///
/// Uses a simplified 360-day year (12 months x 30 days) for deterministic
/// calendar math suitable for fantasy settings.
#[derive(Debug, Clone)]
pub struct SimClock {
    tick: u64,
    start_date: WorldDate,
    hours_per_tick: f64,
    accumulated_hours: f64,
}

impl SimClock {
    /// Create a new clock starting at tick 0 with the given start date and tick duration.
    pub fn new(start_date: WorldDate, hours_per_tick: f64) -> Self {
        Self {
            tick: 0,
            start_date,
            hours_per_tick,
            accumulated_hours: 0.0,
        }
    }

    /// Advance the clock by one tick. Returns the new tick number.
    pub fn advance(&mut self) -> u64 {
        self.tick += 1;
        self.accumulated_hours += self.hours_per_tick;
        self.tick
    }

    /// Return the current tick number.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Current in-world date derived from start date + accumulated hours.
    pub fn current_date(&self) -> WorldDate {
        let total_days = (self.accumulated_hours / 24.0).floor() as i64;

        let start_day = self.start_date.day.unwrap_or(1) as i64 - 1;
        let start_month = self.start_date.month.unwrap_or(1) as i64 - 1;
        let start_year = self.start_date.year;

        let abs_days = start_year * 360 + start_month * 30 + start_day + total_days;

        let year = abs_days.div_euclid(360);
        let remaining = abs_days.rem_euclid(360);
        let month = remaining / 30 + 1;
        let day = remaining % 30 + 1;

        WorldDate {
            year,
            month: Some(month as u32),
            day: Some(day as u32),
            era: self.start_date.era.clone(),
        }
    }

    /// Current hour of the day (0.0..24.0), used by the schedule system.
    pub fn hour_of_day(&self) -> f64 {
        let start_hour = if let Some(d) = self.start_date.day {
            // Start of day
            let _ = d;
            0.0
        } else {
            0.0
        };
        (start_hour + self.accumulated_hours) % 24.0
    }

    /// Total elapsed in-world hours since simulation start.
    pub fn elapsed_hours(&self) -> f64 {
        self.accumulated_hours
    }

    /// Return the configured number of in-world hours per tick.
    pub fn hours_per_tick(&self) -> f64 {
        self.hours_per_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_initial_state() {
        let clock = SimClock::new(WorldDate::new(1), 1.0);
        assert_eq!(clock.tick(), 0);
        assert_eq!(clock.elapsed_hours(), 0.0);
    }

    #[test]
    fn clock_advance_increments() {
        let mut clock = SimClock::new(WorldDate::new(1), 2.0);
        clock.advance();
        clock.advance();
        clock.advance();
        assert_eq!(clock.tick(), 3);
        assert!((clock.elapsed_hours() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clock_hour_of_day_wraps() {
        let mut clock = SimClock::new(WorldDate::new(1), 1.0);
        for _ in 0..25 {
            clock.advance();
        }
        assert!((clock.hour_of_day() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clock_date_advances_across_days() {
        let start = WorldDate {
            year: 1,
            month: Some(1),
            day: Some(1),
            era: None,
        };
        let mut clock = SimClock::new(start, 1.0);
        // Advance 24 ticks = 1 day
        for _ in 0..24 {
            clock.advance();
        }
        let date = clock.current_date();
        assert_eq!(date.year, 1);
        assert_eq!(date.month, Some(1));
        assert_eq!(date.day, Some(2));
    }

    #[test]
    fn clock_date_advances_across_months() {
        let start = WorldDate {
            year: 1,
            month: Some(1),
            day: Some(30),
            era: None,
        };
        let mut clock = SimClock::new(start, 1.0);
        // Advance 24 ticks = 1 day, wrapping from day 30 to month 2 day 1
        for _ in 0..24 {
            clock.advance();
        }
        let date = clock.current_date();
        assert_eq!(date.year, 1);
        assert_eq!(date.month, Some(2));
        assert_eq!(date.day, Some(1));
    }
}
