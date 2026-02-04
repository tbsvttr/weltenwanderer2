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
    Rest,
    Work,
    Eat,
    Socialize,
    Travel(EntityId),
    Patrol,
    Idle,
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
    pub hour_start: f64,
    pub hour_end: f64,
    pub activity: Activity,
}

impl ScheduleEntry {
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
    pub entries: Vec<ScheduleEntry>,
}

impl Schedule {
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
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
            current_activities: HashMap::new(),
            needs_satisfaction_buffer: Vec::new(),
        }
    }

    pub fn set_schedule(&mut self, entity: EntityId, schedule: Schedule) {
        self.schedules.insert(entity, schedule);
    }

    pub fn get_schedule(&self, entity: EntityId) -> Option<&Schedule> {
        self.schedules.get(&entity)
    }

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
        let alive_chars: Vec<EntityId> = ctx
            .world
            .entities_by_kind(&EntityKind::Character)
            .iter()
            .filter(|e| {
                e.components
                    .character
                    .as_ref()
                    .is_some_and(|c| c.status == CharacterStatus::Alive)
            })
            .map(|e| e.id)
            .collect();

        for id in alive_chars {
            self.schedules
                .entry(id)
                .or_insert_with(Schedule::default_npc);
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
}
