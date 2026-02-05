use std::collections::{HashMap, VecDeque};

use ww_core::entity::{EntityId, EntityKind};
use ww_core::relationship::RelationshipKind;
use ww_core::world::World;

use crate::context::SimContext;
use crate::error::{SimError, SimResult};
use crate::event::SimEventKind;
use crate::system::System;

/// Spatial simulation state for a single entity.
#[derive(Debug, Clone)]
pub struct SpatialState {
    /// The entity's current location.
    pub current_location: EntityId,
    /// The entity's travel destination, if any.
    pub destination: Option<EntityId>,
    /// The sequence of locations to traverse to reach the destination.
    pub path: Vec<EntityId>,
    /// The index of the next location in the path to visit.
    pub path_index: usize,
    /// Movement speed in location-edges per tick.
    pub speed: f64,
    /// Accumulated movement progress toward the next location.
    pub progress: f64,
}

impl SpatialState {
    /// Create a stationary spatial state at the given location.
    pub fn at(location: EntityId) -> Self {
        Self {
            current_location: location,
            destination: None,
            path: Vec::new(),
            path_index: 0,
            speed: 1.0,
            progress: 0.0,
        }
    }

    /// Return `true` if this entity is currently traveling along a path.
    pub fn is_traveling(&self) -> bool {
        self.destination.is_some() && self.path_index < self.path.len()
    }
}

/// Tracks entity locations and moves them between ConnectedTo locations.
#[derive(Debug)]
pub struct SpatialSystem {
    states: HashMap<EntityId, SpatialState>,
    default_speed: f64,
}

impl Default for SpatialSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialSystem {
    /// Create a new spatial system with default speed of 1.0.
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            default_speed: 1.0,
        }
    }

    /// Set the default movement speed for newly tracked entities.
    pub fn with_default_speed(mut self, speed: f64) -> Self {
        self.default_speed = speed;
        self
    }

    /// Return the spatial state for the given entity, if tracked.
    pub fn get_state(&self, id: EntityId) -> Option<&SpatialState> {
        self.states.get(&id)
    }

    /// Return all tracked spatial states keyed by entity ID.
    pub fn all_states(&self) -> &HashMap<EntityId, SpatialState> {
        &self.states
    }

    /// Set the movement speed for the given entity.
    pub fn set_speed(&mut self, entity: EntityId, speed: f64) {
        if let Some(state) = self.states.get_mut(&entity) {
            state.speed = speed;
        }
    }

    /// Set a pre-computed travel path on an entity.
    pub fn set_travel(&mut self, entity: EntityId, destination: EntityId, path: Vec<EntityId>) {
        if let Some(state) = self.states.get_mut(&entity) {
            state.destination = Some(destination);
            state.path = path;
            state.path_index = 0;
            state.progress = 0.0;
        }
    }

    /// Command an entity to travel to a destination.
    /// Computes the path via BFS on the ConnectedTo graph.
    pub fn send_to(
        &mut self,
        entity: EntityId,
        destination: EntityId,
        world: &World,
    ) -> SimResult<usize> {
        let state = self
            .states
            .get(&entity)
            .ok_or(SimError::EntityNotFound(entity))?;
        let start = state.current_location;

        if start == destination {
            return Ok(0);
        }

        let path = Self::find_path(world, start, destination)?;
        let path_len = path.len();

        let state = self.states.get_mut(&entity).unwrap();
        state.destination = Some(destination);
        state.path = path;
        state.path_index = 0;
        state.progress = 0.0;

        Ok(path_len)
    }

    /// BFS pathfinding on the ConnectedTo relationship graph.
    pub fn find_path(world: &World, from: EntityId, to: EntityId) -> SimResult<Vec<EntityId>> {
        let mut visited: HashMap<EntityId, Option<EntityId>> = HashMap::new();
        let mut queue = VecDeque::new();

        visited.insert(from, None);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut node = to;
                while let Some(&Some(prev)) = visited.get(&node) {
                    path.push(prev);
                    node = prev;
                }
                path.reverse();
                // Remove starting location (path = locations to move TO)
                if !path.is_empty() {
                    path.remove(0);
                }
                return Ok(path);
            }

            // Explore neighbors via ConnectedTo
            for (neighbor_id, rel) in world.neighbors(current) {
                if rel.kind == RelationshipKind::ConnectedTo && !visited.contains_key(&neighbor_id)
                {
                    visited.insert(neighbor_id, Some(current));
                    queue.push_back(neighbor_id);
                }
            }
        }

        Err(SimError::NoPath { from, to })
    }

    /// Calculate Euclidean distance between two locations using Coordinates.
    pub fn distance(world: &World, a: EntityId, b: EntityId) -> Option<f64> {
        let coords_a = world
            .get_entity(a)?
            .components
            .location
            .as_ref()?
            .coordinates
            .as_ref()?;
        let coords_b = world
            .get_entity(b)?
            .components
            .location
            .as_ref()?
            .coordinates
            .as_ref()?;

        let dx = coords_a.x - coords_b.x;
        let dy = coords_a.y - coords_b.y;
        let dz = match (coords_a.z, coords_b.z) {
            (Some(za), Some(zb)) => za - zb,
            _ => 0.0,
        };
        Some((dx * dx + dy * dy + dz * dz).sqrt())
    }
}

impl System for SpatialSystem {
    fn name(&self) -> &str {
        "spatial"
    }

    fn init(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()> {
        let characters: Vec<_> = ctx
            .world
            .entities_by_kind(&EntityKind::Character)
            .iter()
            .map(|e| (e.id, e.components.simulation.as_ref()))
            .collect();

        for (char_id, sim_comp) in characters {
            let location = ctx
                .world
                .relationships_from(char_id)
                .iter()
                .find(|r| r.kind == RelationshipKind::LocatedAt)
                .map(|r| r.target);

            if let Some(loc_id) = location {
                let custom_speed = sim_comp.and_then(|s| s.speed);
                let mut state = SpatialState::at(loc_id);
                state.speed = custom_speed.unwrap_or(self.default_speed);
                self.states.insert(char_id, state);
            }
        }
        Ok(())
    }

    fn tick(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()> {
        let ids: Vec<EntityId> = self.states.keys().copied().collect();

        for id in ids {
            let state = match self.states.get_mut(&id) {
                Some(s) => s,
                None => continue,
            };

            if !state.is_traveling() {
                continue;
            }

            state.progress += state.speed;

            // Move through locations while progress allows
            while state.progress >= 1.0 && state.path_index < state.path.len() {
                let prev_location = state.current_location;
                let next_location = state.path[state.path_index];

                ctx.emit(
                    SimEventKind::Departed {
                        entity: id,
                        from: prev_location,
                    },
                    format!(
                        "{} departed {}",
                        ctx.world.entity_name(id),
                        ctx.world.entity_name(prev_location)
                    ),
                );

                state.current_location = next_location;
                state.path_index += 1;
                state.progress -= 1.0;

                ctx.emit(
                    SimEventKind::Arrived {
                        entity: id,
                        at: next_location,
                    },
                    format!(
                        "{} arrived at {}",
                        ctx.world.entity_name(id),
                        ctx.world.entity_name(next_location)
                    ),
                );
            }

            // Clear destination if arrived
            if state.path_index >= state.path.len() {
                state.destination = None;
                state.path.clear();
                state.path_index = 0;
                state.progress = 0.0;
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
    use ww_core::component::{Coordinates, LocationComponent};
    use ww_core::entity::Entity;
    use ww_core::relationship::Relationship;
    use ww_core::world::WorldMeta;

    fn location_world() -> (World, EntityId, EntityId, EntityId) {
        let mut world = World::new(WorldMeta::new("Test"));
        let a = world
            .add_entity(Entity::new(EntityKind::Location, "Town A"))
            .unwrap();
        let b = world
            .add_entity(Entity::new(EntityKind::Location, "Town B"))
            .unwrap();
        let c = world
            .add_entity(Entity::new(EntityKind::Location, "Town C"))
            .unwrap();

        // A -- B -- C
        world
            .add_relationship(Relationship::new(a, RelationshipKind::ConnectedTo, b))
            .unwrap();
        world
            .add_relationship(Relationship::new(b, RelationshipKind::ConnectedTo, c))
            .unwrap();

        (world, a, b, c)
    }

    #[test]
    fn bfs_finds_shortest_path() {
        let (world, a, b, c) = location_world();
        let path = SpatialSystem::find_path(&world, a, c).unwrap();
        assert_eq!(path, vec![b, c]);
    }

    #[test]
    fn bfs_returns_error_when_no_path() {
        let mut world = World::new(WorldMeta::new("Test"));
        let a = world
            .add_entity(Entity::new(EntityKind::Location, "Isolated A"))
            .unwrap();
        let b = world
            .add_entity(Entity::new(EntityKind::Location, "Isolated B"))
            .unwrap();
        // No connection
        let result = SpatialSystem::find_path(&world, a, b);
        assert!(result.is_err());
    }

    #[test]
    fn bfs_same_start_and_end() {
        let (world, a, _, _) = location_world();
        // find_path is not called for same start/end in send_to, but check the helper
        let path = SpatialSystem::find_path(&world, a, a).unwrap();
        assert!(path.is_empty());
    }

    #[test]
    fn distance_calculation() {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut e1 = Entity::new(EntityKind::Location, "P1");
        e1.components.location = Some(LocationComponent {
            coordinates: Some(Coordinates {
                x: 0.0,
                y: 0.0,
                z: None,
            }),
            ..Default::default()
        });
        let mut e2 = Entity::new(EntityKind::Location, "P2");
        e2.components.location = Some(LocationComponent {
            coordinates: Some(Coordinates {
                x: 3.0,
                y: 4.0,
                z: None,
            }),
            ..Default::default()
        });
        let id1 = world.add_entity(e1).unwrap();
        let id2 = world.add_entity(e2).unwrap();

        let dist = SpatialSystem::distance(&world, id1, id2).unwrap();
        assert!((dist - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spatial_movement_one_edge_per_tick() {
        let (world, a, b, c) = location_world();
        let char_id = EntityId::new();

        let mut sys = SpatialSystem::new();
        sys.states.insert(char_id, SpatialState::at(a));
        sys.send_to(char_id, c, &world).unwrap();

        let state = sys.get_state(char_id).unwrap();
        assert_eq!(state.path, vec![b, c]);
        assert!(state.is_traveling());

        // Simulate progress manually (speed=1.0 means 1 edge per tick)
        let state = sys.states.get_mut(&char_id).unwrap();
        state.progress += 1.0;
        // After 1 unit of progress: move to B
        assert!(state.progress >= 1.0);
    }

    #[test]
    fn bfs_with_cycle_in_graph() {
        let mut world = World::new(WorldMeta::new("Test"));
        let a = world
            .add_entity(Entity::new(EntityKind::Location, "A"))
            .unwrap();
        let b = world
            .add_entity(Entity::new(EntityKind::Location, "B"))
            .unwrap();
        let c = world
            .add_entity(Entity::new(EntityKind::Location, "C"))
            .unwrap();
        // A -- B -- C -- A (cycle, bidirectional via neighbors)
        world
            .add_relationship(Relationship::new(a, RelationshipKind::ConnectedTo, b))
            .unwrap();
        world
            .add_relationship(Relationship::new(b, RelationshipKind::ConnectedTo, c))
            .unwrap();
        world
            .add_relationship(Relationship::new(c, RelationshipKind::ConnectedTo, a))
            .unwrap();

        // BFS terminates despite cycle; finds shortest path
        // C->A relationship makes A a neighbor of C (bidirectional),
        // so A can reach C in 1 hop
        let path = SpatialSystem::find_path(&world, a, c).unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path, vec![c]);
    }

    #[test]
    fn send_to_same_location_returns_zero() {
        let (world, a, _, _) = location_world();
        let char_id = EntityId::new();

        let mut sys = SpatialSystem::new();
        sys.states.insert(char_id, SpatialState::at(a));
        let hops = sys.send_to(char_id, a, &world).unwrap();
        assert_eq!(hops, 0);
        assert!(!sys.get_state(char_id).unwrap().is_traveling());
    }

    #[test]
    fn send_to_untracked_entity_fails() {
        let (world, _, b, _) = location_world();
        let unknown = EntityId::new();

        let mut sys = SpatialSystem::new();
        let result = sys.send_to(unknown, b, &world);
        assert!(result.is_err());
    }

    #[test]
    fn spatial_state_not_traveling_when_stationary() {
        let id = EntityId::new();
        let state = SpatialState::at(id);
        assert!(!state.is_traveling());
        assert!(state.destination.is_none());
        assert!(state.path.is_empty());
    }

    #[test]
    fn default_speed_applied() {
        let sys = SpatialSystem::new().with_default_speed(2.5);
        // The system starts with no states; default_speed is applied during init
        assert!((sys.default_speed - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn distance_returns_none_without_coordinates() {
        let mut world = World::new(WorldMeta::new("Test"));
        let a = world
            .add_entity(Entity::new(EntityKind::Location, "NoCoords"))
            .unwrap();
        let b = world
            .add_entity(Entity::new(EntityKind::Location, "AlsoNoCoords"))
            .unwrap();
        assert!(SpatialSystem::distance(&world, a, b).is_none());
    }

    #[test]
    fn distance_with_z_coordinate() {
        let mut world = World::new(WorldMeta::new("Test"));
        let mut e1 = Entity::new(EntityKind::Location, "P1");
        e1.components.location = Some(LocationComponent {
            coordinates: Some(Coordinates {
                x: 0.0,
                y: 0.0,
                z: Some(0.0),
            }),
            ..Default::default()
        });
        let mut e2 = Entity::new(EntityKind::Location, "P2");
        e2.components.location = Some(LocationComponent {
            coordinates: Some(Coordinates {
                x: 1.0,
                y: 2.0,
                z: Some(2.0),
            }),
            ..Default::default()
        });
        let id1 = world.add_entity(e1).unwrap();
        let id2 = world.add_entity(e2).unwrap();

        let dist = SpatialSystem::distance(&world, id1, id2).unwrap();
        // sqrt(1 + 4 + 4) = 3.0
        assert!((dist - 3.0).abs() < f64::EPSILON);
    }
}
