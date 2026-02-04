use rand::SeedableRng;
use rand::rngs::StdRng;
use ww_core::world::World;

use crate::clock::SimClock;
use crate::config::SimConfig;
use crate::context::SimContext;
use crate::error::SimResult;
use crate::event::EventLog;
use crate::needs::NeedsSystem;
use crate::schedule::ScheduleSystem;
use crate::system::System;

/// The top-level simulation orchestrator.
///
/// Owns the world, clock, RNG, event log, and registered systems.
/// Drives the tick loop and coordinates cross-system effects.
pub struct Simulation {
    world: World,
    clock: SimClock,
    rng: StdRng,
    events: EventLog,
    systems: Vec<Box<dyn System>>,
    initialized: bool,
}

impl std::fmt::Debug for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Simulation")
            .field("tick", &self.clock.tick())
            .field("systems", &self.systems.len())
            .field("events", &self.events.len())
            .finish()
    }
}

impl Simulation {
    /// Create a new simulation from a world and configuration.
    pub fn new(world: World, config: SimConfig) -> Self {
        let clock = SimClock::new(config.start_date, config.hours_per_tick);
        let rng = StdRng::seed_from_u64(config.seed);
        let events = EventLog::new(config.max_events);
        Self {
            world,
            clock,
            rng,
            events,
            systems: Vec::new(),
            initialized: false,
        }
    }

    /// Register a system. Systems are ticked in registration order.
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Initialize all registered systems.
    pub fn init(&mut self) -> SimResult<()> {
        if self.initialized {
            return Ok(());
        }
        for i in 0..self.systems.len() {
            let mut system = std::mem::replace(&mut self.systems[i], Box::new(NoopSystem));
            let mut ctx = SimContext {
                world: &mut self.world,
                clock: &self.clock,
                events: &mut self.events,
                rng: &mut self.rng,
            };
            system.init(&mut ctx)?;
            self.systems[i] = system;
        }
        self.initialized = true;
        Ok(())
    }

    /// Advance the simulation by one tick.
    pub fn tick(&mut self) -> SimResult<()> {
        if !self.initialized {
            self.init()?;
        }

        self.clock.advance();

        for i in 0..self.systems.len() {
            let mut system = std::mem::replace(&mut self.systems[i], Box::new(NoopSystem));
            let mut ctx = SimContext {
                world: &mut self.world,
                clock: &self.clock,
                events: &mut self.events,
                rng: &mut self.rng,
            };
            system.tick(&mut ctx)?;
            self.systems[i] = system;
        }

        self.apply_cross_system_effects();
        Ok(())
    }

    /// Advance the simulation by `n` ticks.
    pub fn run(&mut self, n: u64) -> SimResult<()> {
        for _ in 0..n {
            self.tick()?;
        }
        Ok(())
    }

    /// Apply cross-system effects after all systems have ticked.
    fn apply_cross_system_effects(&mut self) {
        // Collect schedule -> needs satisfaction effects
        let mut effects = Vec::new();
        for system in &mut self.systems {
            if let Some(schedule) = system.as_any_mut().downcast_mut::<ScheduleSystem>() {
                effects = schedule.drain_need_effects();
                break;
            }
        }

        // Apply to NeedsSystem
        if !effects.is_empty() {
            for system in &mut self.systems {
                if let Some(needs) = system.as_any_mut().downcast_mut::<NeedsSystem>() {
                    for (entity, need, amount) in &effects {
                        if let Some(state) = needs.get_state_mut(*entity) {
                            state.satisfy(need, *amount);
                        }
                    }
                    break;
                }
            }
        }
    }

    /// Return a reference to the simulation world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Return a mutable reference to the simulation world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Return a reference to the simulation clock.
    pub fn clock(&self) -> &SimClock {
        &self.clock
    }

    /// Return a reference to the simulation event log.
    pub fn events(&self) -> &EventLog {
        &self.events
    }

    /// Access a system by downcasting to a concrete type.
    pub fn get_system<T: System + 'static>(&self) -> Option<&T> {
        self.systems
            .iter()
            .find_map(|s| s.as_any().downcast_ref::<T>())
    }

    /// Access a system mutably by downcasting to a concrete type.
    pub fn get_system_mut<T: System + 'static>(&mut self) -> Option<&mut T> {
        self.systems
            .iter_mut()
            .find_map(|s| s.as_any_mut().downcast_mut::<T>())
    }

    /// Extract the world, consuming the simulation.
    pub fn into_world(self) -> World {
        self.world
    }

    /// Return the current simulation tick number.
    pub fn current_tick(&self) -> u64 {
        self.clock.tick()
    }
}

/// Placeholder system used during the swap-and-tick pattern.
#[derive(Debug)]
struct NoopSystem;

impl System for NoopSystem {
    fn name(&self) -> &str {
        "noop"
    }
    fn tick(&mut self, _ctx: &mut SimContext<'_>) -> SimResult<()> {
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
    use crate::needs::{NeedsConfig, NeedsSystem};
    use crate::schedule::ScheduleSystem;
    use crate::spatial::SpatialSystem;
    use ww_core::component::{CharacterComponent, CharacterStatus};
    use ww_core::entity::{Entity, EntityKind};
    use ww_core::relationship::{Relationship, RelationshipKind};
    use ww_core::world::WorldMeta;

    fn test_world_with_character() -> (World, ww_core::entity::EntityId) {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut kael = Entity::new(EntityKind::Character, "Kael");
        kael.components.character = Some(CharacterComponent {
            status: CharacterStatus::Alive,
            ..Default::default()
        });
        let id = world.add_entity(kael).unwrap();
        (world, id)
    }

    #[test]
    fn full_tick_integration() {
        let (world, _id) = test_world_with_character();
        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::with_default_config());
        sim.add_system(ScheduleSystem::new());
        sim.add_system(SpatialSystem::new());

        sim.run(24).unwrap();

        assert_eq!(sim.current_tick(), 24);
        // Should have activity changed events (at least the initial assignment)
        assert!(!sim.events().is_empty());
    }

    #[test]
    fn custom_system_registration() {
        #[derive(Debug)]
        struct CustomSystem {
            ticked: bool,
        }
        impl System for CustomSystem {
            fn name(&self) -> &str {
                "custom"
            }
            fn tick(&mut self, _ctx: &mut SimContext<'_>) -> SimResult<()> {
                self.ticked = true;
                Ok(())
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        let world = World::new(WorldMeta::new("Test"));
        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(CustomSystem { ticked: false });

        sim.tick().unwrap();

        let custom = sim.get_system::<CustomSystem>().unwrap();
        assert!(custom.ticked);
    }

    #[test]
    fn deterministic_rng() {
        let make_sim = || {
            let (world, _) = test_world_with_character();
            let mut sim = Simulation::new(world, SimConfig::default().with_seed(123));
            sim.add_system(NeedsSystem::with_default_config());
            sim.add_system(ScheduleSystem::new());
            sim.run(10).unwrap();
            sim.events()
                .events()
                .iter()
                .map(|e| e.description.clone())
                .collect::<Vec<_>>()
        };

        let run1 = make_sim();
        let run2 = make_sim();
        assert_eq!(run1, run2);
    }

    #[test]
    fn into_world_preserves_changes() {
        let (world, id) = test_world_with_character();
        let mut config = NeedsConfig {
            death_threshold: 0.0,
            ..NeedsConfig::default()
        };
        config.decay_rates.values_mut().for_each(|v| *v = 1.0); // Die quickly

        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::new(config));
        sim.run(5).unwrap();

        let world = sim.into_world();
        // Character should be dead
        let entity = world.get_entity(id).unwrap();
        assert_eq!(
            entity.components.character.as_ref().unwrap().status,
            CharacterStatus::Dead
        );
    }

    #[test]
    fn empty_world_no_crash() {
        let world = World::new(WorldMeta::new("Empty"));
        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::with_default_config());
        sim.add_system(ScheduleSystem::new());
        sim.add_system(SpatialSystem::new());
        sim.run(100).unwrap();
        assert_eq!(sim.current_tick(), 100);
    }

    #[test]
    fn needs_system_decays_each_tick() {
        let (world, id) = test_world_with_character();
        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::with_default_config());

        sim.run(10).unwrap();

        let needs = sim.get_system::<NeedsSystem>().unwrap();
        let state = needs.get_state(id).unwrap();
        // After 10 ticks at 0.01 decay, hunger should be around 0.9
        let hunger = state.get(&crate::needs::NeedKind::Hunger).unwrap();
        assert!((hunger - 0.9).abs() < 0.01);
    }

    #[test]
    fn schedule_satisfies_needs_via_orchestrator() {
        let (world, id) = test_world_with_character();
        // Start at hour 0 (rest time) with 1 hour/tick
        let config = SimConfig::default();
        let mut sim = Simulation::new(world, config);
        sim.add_system(NeedsSystem::with_default_config());
        sim.add_system(ScheduleSystem::new());

        sim.run(1).unwrap();

        let needs = sim.get_system::<NeedsSystem>().unwrap();
        let state = needs.get_state(id).unwrap();
        // Rest activity satisfies rest need by 0.08, but also decays 0.01
        // So rest should be: 1.0 - 0.01 + 0.08 = clamped to 1.0
        let rest = state.get(&crate::needs::NeedKind::Rest).unwrap();
        assert!(rest > 0.95, "rest should be high due to satisfaction");
    }

    #[test]
    fn spatial_system_movement() {
        let mut world = World::new(WorldMeta::new("Test"));
        let loc_a = world
            .add_entity(Entity::new(EntityKind::Location, "A"))
            .unwrap();
        let loc_b = world
            .add_entity(Entity::new(EntityKind::Location, "B"))
            .unwrap();
        world
            .add_relationship(Relationship::new(
                loc_a,
                RelationshipKind::ConnectedTo,
                loc_b,
            ))
            .unwrap();

        let mut kael = Entity::new(EntityKind::Character, "Kael");
        kael.components.character = Some(CharacterComponent {
            status: CharacterStatus::Alive,
            ..Default::default()
        });
        let kael_id = world.add_entity(kael).unwrap();
        world
            .add_relationship(Relationship::new(
                kael_id,
                RelationshipKind::LocatedAt,
                loc_a,
            ))
            .unwrap();

        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(SpatialSystem::new());
        sim.init().unwrap();

        // Compute path before getting mutable system access
        let path = SpatialSystem::find_path(sim.world(), loc_a, loc_b).unwrap();
        assert_eq!(path.len(), 1);

        // Set travel state on the spatial system
        let spatial = sim.get_system_mut::<SpatialSystem>().unwrap();
        spatial.set_travel(kael_id, loc_b, path);

        sim.tick().unwrap();

        let spatial = sim.get_system::<SpatialSystem>().unwrap();
        let state = spatial.get_state(kael_id).unwrap();
        assert_eq!(state.current_location, loc_b);
        assert!(!state.is_traveling());
    }

    #[test]
    fn init_is_idempotent() {
        let (world, _) = test_world_with_character();
        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::with_default_config());

        sim.init().unwrap();
        sim.init().unwrap(); // Should be a no-op
        assert_eq!(sim.current_tick(), 0);
    }

    #[test]
    fn multiple_characters_simulated() {
        let mut world = World::new(WorldMeta::new("Test"));
        for name in ["Alice", "Bob", "Carol"] {
            let mut ch = Entity::new(EntityKind::Character, name);
            ch.components.character = Some(CharacterComponent {
                status: CharacterStatus::Alive,
                ..Default::default()
            });
            world.add_entity(ch).unwrap();
        }

        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::with_default_config());
        sim.add_system(ScheduleSystem::new());
        sim.run(24).unwrap();

        // All 3 characters should have need states
        let needs = sim.get_system::<NeedsSystem>().unwrap();
        assert_eq!(needs.all_states().len(), 3);

        // All 3 should have schedules
        let sched = sim.get_system::<ScheduleSystem>().unwrap();
        for id in needs.all_states().keys() {
            assert!(sched.get_schedule(*id).is_some());
        }
    }

    #[test]
    fn death_emits_entity_died_event() {
        let (world, id) = test_world_with_character();
        let config = NeedsConfig {
            death_threshold: 0.0,
            decay_rates: {
                let mut rates = std::collections::HashMap::new();
                // Hunger decays very fast, others normal
                rates.insert(crate::needs::NeedKind::Hunger, 0.5);
                rates.insert(crate::needs::NeedKind::Rest, 0.01);
                rates.insert(crate::needs::NeedKind::Social, 0.01);
                rates.insert(crate::needs::NeedKind::Safety, 0.01);
                rates
            },
            ..NeedsConfig::default()
        };

        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::new(config));
        sim.run(5).unwrap();

        // Should have a death event
        let death_events: Vec<_> = sim
            .events()
            .events()
            .iter()
            .filter(|e| matches!(&e.kind, crate::event::SimEventKind::EntityDied { entity, .. } if *entity == id))
            .collect();
        assert!(
            !death_events.is_empty(),
            "expected at least one EntityDied event"
        );
    }

    #[test]
    fn event_log_tracks_all_system_types() {
        let (world, _id) = test_world_with_character();
        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::with_default_config());
        sim.add_system(ScheduleSystem::new());
        sim.run(100).unwrap();

        let events = sim.events().events();
        // Should have ActivityChanged events from schedule system
        let has_activity = events
            .iter()
            .any(|e| matches!(&e.kind, crate::event::SimEventKind::ActivityChanged { .. }));
        assert!(has_activity, "expected ActivityChanged events");

        // Should have NeedCritical events from needs system (after 100 ticks some needs decay)
        let has_critical = events
            .iter()
            .any(|e| matches!(&e.kind, crate::event::SimEventKind::NeedCritical { .. }));
        assert!(has_critical, "expected NeedCritical events");
    }

    #[test]
    fn config_max_events_limits_log_in_simulation() {
        let (world, _) = test_world_with_character();
        let config = SimConfig::default().with_max_events(5);
        let mut sim = Simulation::new(world, config);
        sim.add_system(NeedsSystem::with_default_config());
        sim.add_system(ScheduleSystem::new());
        sim.run(100).unwrap();

        // Event log should be capped at 5
        assert!(
            sim.events().len() <= 5,
            "event log should be limited to 5 events, got {}",
            sim.events().len()
        );
    }

    #[test]
    fn dead_characters_stop_decaying() {
        let (world, id) = test_world_with_character();
        let config = NeedsConfig {
            death_threshold: 0.0,
            decay_rates: {
                let mut rates = std::collections::HashMap::new();
                rates.insert(crate::needs::NeedKind::Hunger, 1.0); // Dies on tick 1
                rates.insert(crate::needs::NeedKind::Rest, 0.01);
                rates.insert(crate::needs::NeedKind::Social, 0.01);
                rates.insert(crate::needs::NeedKind::Safety, 0.01);
                rates
            },
            ..NeedsConfig::default()
        };

        let mut sim = Simulation::new(world, SimConfig::default());
        sim.add_system(NeedsSystem::new(config));
        sim.run(10).unwrap();

        // Character should be dead
        let entity = sim.world().get_entity(id).unwrap();
        assert_eq!(
            entity.components.character.as_ref().unwrap().status,
            CharacterStatus::Dead
        );

        // Rest should not have decayed much past the death tick (only tick 1 alive)
        let needs = sim.get_system::<NeedsSystem>().unwrap();
        let state = needs.get_state(id).unwrap();
        let rest = state.get(&crate::needs::NeedKind::Rest).unwrap();
        // Only 1 tick of decay at 0.01 = 0.99
        assert!(rest > 0.95, "rest should barely have decayed: {rest}");
    }

    #[test]
    fn get_system_returns_none_for_unregistered() {
        let world = World::new(WorldMeta::new("Test"));
        let sim = Simulation::new(world, SimConfig::default());
        assert!(sim.get_system::<NeedsSystem>().is_none());
        assert!(sim.get_system::<ScheduleSystem>().is_none());
        assert!(sim.get_system::<SpatialSystem>().is_none());
    }

    #[test]
    fn simulation_debug_format() {
        let world = World::new(WorldMeta::new("Test"));
        let sim = Simulation::new(world, SimConfig::default());
        let debug = format!("{sim:?}");
        assert!(debug.contains("Simulation"));
        assert!(debug.contains("tick"));
    }
}
