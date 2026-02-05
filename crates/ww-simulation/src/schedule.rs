use std::collections::HashMap;

use ww_core::component::CharacterStatus;
use ww_core::entity::{EntityId, EntityKind};

use crate::context::SimContext;
use crate::error::SimResult;
use crate::event::SimEventKind;
use crate::needs::NeedKind;
use crate::system::System;

/// An activity that an NPC can perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Activity {
    /// Sleeping or resting to recover energy.
    Rest,
    /// Performing daily work or labor.
    Work,
    /// Eating a meal to satisfy hunger.
    Eat,
    /// Socializing with other characters.
    Socialize,
    /// Traveling to a destination location.
    Travel(EntityId),
    /// Patrolling an area for safety.
    Patrol,
    /// Doing nothing in particular.
    Idle,
    /// A user-defined activity.
    Custom(String),
}

impl std::fmt::Display for Activity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rest => write!(f, "rest"),
            Self::Work => write!(f, "work"),
            Self::Eat => write!(f, "eat"),
            Self::Socialize => write!(f, "socialize"),
            Self::Travel(id) => write!(f, "travel to {id}"),
            Self::Patrol => write!(f, "patrol"),
            Self::Idle => write!(f, "idle"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// A single schedule entry: from hour_start to hour_end, do activity.
#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    /// The starting hour of this time slot (0.0..24.0).
    pub hour_start: f64,
    /// The ending hour of this time slot (0.0..24.0); may be less than start for midnight wrap.
    pub hour_end: f64,
    /// The activity to perform during this time slot.
    pub activity: Activity,
}

impl ScheduleEntry {
    /// Create a new schedule entry for the given time window and activity.
    pub fn new(hour_start: f64, hour_end: f64, activity: Activity) -> Self {
        Self {
            hour_start,
            hour_end,
            activity,
        }
    }

    /// Check if a given hour falls within this entry's time window.
    pub fn contains_hour(&self, hour: f64) -> bool {
        if self.hour_start <= self.hour_end {
            hour >= self.hour_start && hour < self.hour_end
        } else {
            // Wraps past midnight
            hour >= self.hour_start || hour < self.hour_end
        }
    }
}

/// A daily schedule: ordered list of time-slotted activities.
#[derive(Debug, Clone)]
pub struct Schedule {
    /// The ordered list of time-slotted activity entries.
    pub entries: Vec<ScheduleEntry>,
}

impl Schedule {
    /// Create a new schedule from the given list of time-slotted entries.
    pub fn new(entries: Vec<ScheduleEntry>) -> Self {
        Self { entries }
    }

    /// Find the activity for a given hour of day. Returns Idle if no entry matches.
    pub fn activity_at(&self, hour: f64) -> &Activity {
        self.entries
            .iter()
            .find(|e| e.contains_hour(hour))
            .map(|e| &e.activity)
            .unwrap_or(&Activity::Idle)
    }

    /// Default NPC schedule template.
    pub fn default_npc() -> Self {
        Self::new(vec![
            ScheduleEntry::new(22.0, 6.0, Activity::Rest), // Sleep 10pm-6am
            ScheduleEntry::new(6.0, 7.0, Activity::Eat),   // Breakfast
            ScheduleEntry::new(7.0, 12.0, Activity::Work), // Morning work
            ScheduleEntry::new(12.0, 13.0, Activity::Eat), // Lunch
            ScheduleEntry::new(13.0, 17.0, Activity::Work), // Afternoon work
            ScheduleEntry::new(17.0, 18.0, Activity::Eat), // Dinner
            ScheduleEntry::new(18.0, 22.0, Activity::Socialize), // Evening
        ])
    }

    /// Create a schedule from SimScheduleEntry list (from DSL).
    pub fn from_sim_entries(entries: &[ww_core::component::SimScheduleEntry]) -> Self {
        let sched_entries = entries
            .iter()
            .map(|e| {
                let activity = match e.activity.to_lowercase().as_str() {
                    "rest" => Activity::Rest,
                    "work" => Activity::Work,
                    "eat" => Activity::Eat,
                    "socialize" => Activity::Socialize,
                    "patrol" => Activity::Patrol,
                    "idle" => Activity::Idle,
                    _ => Activity::Custom(e.activity.clone()),
                };
                ScheduleEntry::new(e.start_hour, e.end_hour, activity)
            })
            .collect();
        Self::new(sched_entries)
    }
}

/// Maps activities to need satisfaction effects per tick.
pub fn activity_need_effects(activity: &Activity) -> Vec<(NeedKind, f64)> {
    match activity {
        Activity::Eat => vec![(NeedKind::Hunger, 0.15)],
        Activity::Rest => vec![(NeedKind::Rest, 0.08)],
        Activity::Socialize => vec![(NeedKind::Social, 0.05)],
        Activity::Patrol => vec![(NeedKind::Safety, 0.03)],
        Activity::Work | Activity::Travel(_) | Activity::Idle | Activity::Custom(_) => vec![],
    }
}

/// The schedule simulation system.
///
/// Each tick, determines the current activity for every tracked entity
/// based on in-world hour. Emits ActivityChanged events and buffers
/// need satisfaction effects for the orchestrator.
#[derive(Debug)]
pub struct ScheduleSystem {
    schedules: HashMap<EntityId, Schedule>,
    current_activities: HashMap<EntityId, Activity>,
    needs_satisfaction_buffer: Vec<(EntityId, NeedKind, f64)>,
}

impl Default for ScheduleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleSystem {
    /// Create a new empty schedule system.
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
            current_activities: HashMap::new(),
            needs_satisfaction_buffer: Vec::new(),
        }
    }

    /// Assign a daily schedule to the given entity.
    pub fn set_schedule(&mut self, entity: EntityId, schedule: Schedule) {
        self.schedules.insert(entity, schedule);
    }

    /// Return the schedule assigned to the given entity, if any.
    pub fn get_schedule(&self, entity: EntityId) -> Option<&Schedule> {
        self.schedules.get(&entity)
    }

    /// Return the current activity for the given entity, if tracked.
    pub fn current_activity(&self, entity: EntityId) -> Option<&Activity> {
        self.current_activities.get(&entity)
    }

    /// Returns buffered need satisfaction effects from the last tick.
    /// Consumed by the Simulation orchestrator.
    pub fn drain_need_effects(&mut self) -> Vec<(EntityId, NeedKind, f64)> {
        std::mem::take(&mut self.needs_satisfaction_buffer)
    }
}

impl System for ScheduleSystem {
    fn name(&self) -> &str {
        "schedule"
    }

    fn init(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()> {
        let alive_chars: Vec<_> = ctx
            .world
            .entities_by_kind(&EntityKind::Character)
            .iter()
            .filter(|e| {
                e.components
                    .character
                    .as_ref()
                    .is_some_and(|c| c.status == CharacterStatus::Alive)
            })
            .map(|e| (e.id, e.components.simulation.as_ref()))
            .collect();

        for (id, sim_comp) in alive_chars {
            let schedule = sim_comp
                .and_then(|s| s.schedule.as_ref())
                .map(|entries| Schedule::from_sim_entries(entries))
                .unwrap_or_else(Schedule::default_npc);
            self.schedules.insert(id, schedule);
        }
        Ok(())
    }

    fn tick(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()> {
        let hour = ctx.hour_of_day();
        let ids: Vec<EntityId> = self.schedules.keys().copied().collect();

        for id in ids {
            let schedule = match self.schedules.get(&id) {
                Some(s) => s,
                None => continue,
            };
            let new_activity = schedule.activity_at(hour).clone();

            // Detect activity change
            let changed = self.current_activities.get(&id) != Some(&new_activity);
            if changed {
                let old = self
                    .current_activities
                    .insert(id, new_activity.clone())
                    .unwrap_or(Activity::Idle);
                ctx.emit(
                    SimEventKind::ActivityChanged {
                        entity: id,
                        from: old.to_string(),
                        to: new_activity.to_string(),
                    },
                    format!("{} now: {}", ctx.world.entity_name(id), new_activity),
                );
            }

            // Buffer need satisfaction effects
            let current = self
                .current_activities
                .get(&id)
                .cloned()
                .unwrap_or(Activity::Idle);
            for (need, amount) in activity_need_effects(&current) {
                self.needs_satisfaction_buffer.push((id, need, amount));
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_entry_contains_hour() {
        let entry = ScheduleEntry::new(7.0, 12.0, Activity::Work);
        assert!(entry.contains_hour(7.0));
        assert!(entry.contains_hour(10.0));
        assert!(!entry.contains_hour(12.0));
        assert!(!entry.contains_hour(6.0));
    }

    #[test]
    fn schedule_entry_midnight_wrap() {
        let entry = ScheduleEntry::new(22.0, 6.0, Activity::Rest);
        assert!(entry.contains_hour(23.0));
        assert!(entry.contains_hour(2.0));
        assert!(!entry.contains_hour(10.0));
        assert!(!entry.contains_hour(6.0));
    }

    #[test]
    fn default_schedule_activities() {
        let sched = Schedule::default_npc();
        assert_eq!(sched.activity_at(2.0), &Activity::Rest);
        assert_eq!(sched.activity_at(8.0), &Activity::Work);
        assert_eq!(sched.activity_at(6.5), &Activity::Eat);
        assert_eq!(sched.activity_at(19.0), &Activity::Socialize);
    }

    #[test]
    fn activity_need_effects_correct() {
        let effects = activity_need_effects(&Activity::Eat);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, NeedKind::Hunger);

        let effects = activity_need_effects(&Activity::Rest);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, NeedKind::Rest);

        assert!(activity_need_effects(&Activity::Work).is_empty());
    }

    #[test]
    fn custom_schedule() {
        let sched = Schedule::new(vec![
            ScheduleEntry::new(0.0, 12.0, Activity::Work),
            ScheduleEntry::new(12.0, 24.0, Activity::Rest),
        ]);
        assert_eq!(sched.activity_at(6.0), &Activity::Work);
        assert_eq!(sched.activity_at(18.0), &Activity::Rest);
    }

    #[test]
    fn activity_at_returns_idle_for_gaps() {
        // Schedule with a gap between 10 and 14
        let sched = Schedule::new(vec![
            ScheduleEntry::new(6.0, 10.0, Activity::Work),
            ScheduleEntry::new(14.0, 22.0, Activity::Socialize),
        ]);
        assert_eq!(sched.activity_at(8.0), &Activity::Work);
        assert_eq!(sched.activity_at(12.0), &Activity::Idle); // Gap
        assert_eq!(sched.activity_at(16.0), &Activity::Socialize);
    }

    #[test]
    fn default_schedule_full_day_coverage() {
        let sched = Schedule::default_npc();
        // Check that every hour of the day has an activity (not Idle)
        let mut hour = 0.0;
        while hour < 24.0 {
            assert_ne!(
                sched.activity_at(hour),
                &Activity::Idle,
                "hour {hour} should not be idle"
            );
            hour += 0.5;
        }
    }

    #[test]
    fn activity_display() {
        assert_eq!(format!("{}", Activity::Rest), "rest");
        assert_eq!(format!("{}", Activity::Work), "work");
        assert_eq!(format!("{}", Activity::Eat), "eat");
        assert_eq!(format!("{}", Activity::Socialize), "socialize");
        assert_eq!(format!("{}", Activity::Patrol), "patrol");
        assert_eq!(format!("{}", Activity::Idle), "idle");
        assert_eq!(
            format!("{}", Activity::Custom("meditate".into())),
            "meditate"
        );
    }

    #[test]
    fn patrol_satisfies_safety() {
        let effects = activity_need_effects(&Activity::Patrol);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, NeedKind::Safety);
    }

    #[test]
    fn socialize_satisfies_social() {
        let effects = activity_need_effects(&Activity::Socialize);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, NeedKind::Social);
    }
}
