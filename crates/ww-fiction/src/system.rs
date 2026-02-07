//! Fiction system for simulation integration.
//!
//! Generates narrative text from simulation events when running in "watch mode".

use std::any::Any;

use ww_simulation::{SimContext, SimEventKind, SimResult, System};

use crate::narrator::TemplateRegistry;

/// A simulation system that generates narrative from simulation events.
#[derive(Debug)]
pub struct FictionSystem {
    narrator: TemplateRegistry,
    output_buffer: Vec<String>,
    last_processed_count: usize,
}

impl FictionSystem {
    /// Create a new fiction system with default narrator.
    pub fn new() -> Self {
        Self {
            narrator: TemplateRegistry::default(),
            output_buffer: Vec::new(),
            last_processed_count: 0,
        }
    }

    /// Create a fiction system with custom narrator config.
    pub fn with_narrator(narrator: TemplateRegistry) -> Self {
        Self {
            narrator,
            output_buffer: Vec::new(),
            last_processed_count: 0,
        }
    }

    /// Drain the output buffer and return all generated text.
    pub fn drain_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.output_buffer)
    }

    /// Get the narrator.
    pub fn narrator(&self) -> &TemplateRegistry {
        &self.narrator
    }
}

impl Default for FictionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for FictionSystem {
    fn name(&self) -> &str {
        "fiction"
    }

    fn init(&mut self, _ctx: &mut SimContext<'_>) -> SimResult<()> {
        self.last_processed_count = 0;
        Ok(())
    }

    fn tick(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()> {
        // Process only new events since last tick
        let all_events = ctx.events.events();
        let new_events = &all_events[self.last_processed_count..];

        for event in new_events {
            let text = match &event.kind {
                SimEventKind::ActivityChanged { entity, to, .. } => ctx
                    .world
                    .get_entity(*entity)
                    .map(|e| format!("{} begins to {}.", e.name, to)),
                SimEventKind::NeedCritical { entity, need } => ctx
                    .world
                    .get_entity(*entity)
                    .map(|e| format!("{}'s {} has become critical!", e.name, need)),
                SimEventKind::NeedDepleted { entity, need } => ctx
                    .world
                    .get_entity(*entity)
                    .map(|e| format!("{}'s {} is completely depleted!", e.name, need)),
                SimEventKind::NeedSatisfied { entity, need } => ctx
                    .world
                    .get_entity(*entity)
                    .map(|e| format!("{}'s {} has been satisfied.", e.name, need)),
                SimEventKind::Arrived { entity, at } => {
                    let entity_name = ctx.world.get_entity(*entity).map(|e| e.name.as_str());
                    let at_name = ctx.world.get_entity(*at).map(|e| e.name.as_str());
                    if let (Some(who), Some(where_at)) = (entity_name, at_name) {
                        Some(format!("{who} arrives at {where_at}."))
                    } else {
                        None
                    }
                }
                SimEventKind::Departed { entity, from } => {
                    let entity_name = ctx.world.get_entity(*entity).map(|e| e.name.as_str());
                    let from_name = ctx.world.get_entity(*from).map(|e| e.name.as_str());
                    if let (Some(who), Some(where_from)) = (entity_name, from_name) {
                        Some(format!("{who} departs from {where_from}."))
                    } else {
                        None
                    }
                }
                SimEventKind::EntityDied { entity, cause } => ctx
                    .world
                    .get_entity(*entity)
                    .map(|e| format!("{} has perished ({cause}).", e.name)),
                SimEventKind::Custom { label, .. } => Some(label.clone()),
            };

            if let Some(t) = text {
                self.output_buffer.push(t);
            }
        }

        self.last_processed_count = all_events.len();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::{Entity, EntityKind, World, WorldMeta};
    use ww_simulation::{SimConfig, Simulation};

    #[test]
    fn fiction_system_name() {
        let system = FictionSystem::new();
        assert_eq!(system.name(), "fiction");
    }

    #[test]
    fn fiction_system_drain_output() {
        let mut system = FictionSystem::new();
        assert!(system.drain_output().is_empty());

        system.output_buffer.push("Test output.".to_string());
        let output = system.drain_output();
        assert_eq!(output.len(), 1);
        assert!(system.drain_output().is_empty());
    }

    #[test]
    fn fiction_system_in_simulation() {
        let mut world = World::new(WorldMeta::new("Test"));
        let loc = Entity::new(EntityKind::Location, "Village");
        let _loc_id = world.add_entity(loc).unwrap();

        let config = SimConfig::default();
        let mut sim = Simulation::new(world, config);
        sim.add_system(FictionSystem::new());

        // Should not panic
        sim.tick().unwrap();
    }
}
