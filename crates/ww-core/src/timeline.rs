use crate::component::WorldDate;
use crate::entity::{Entity, EntityId, EntityKind};
use crate::world::World;

/// A timeline entry: an event entity with its resolved date.
#[derive(Debug)]
pub struct TimelineEntry<'w> {
    pub entity: &'w Entity,
    pub date: &'w WorldDate,
}

/// Query and iterate events in chronological order.
pub struct Timeline<'w> {
    entries: Vec<TimelineEntry<'w>>,
}

impl<'w> Timeline<'w> {
    /// Build a timeline from all event entities in the world that have a date.
    pub fn from_world(world: &'w World) -> Self {
        let mut entries: Vec<TimelineEntry<'w>> = world
            .all_entities()
            .filter(|e| e.kind == EntityKind::Event)
            .filter_map(|e| {
                e.components
                    .event
                    .as_ref()
                    .and_then(|ec| ec.date.as_ref())
                    .map(|date| TimelineEntry { entity: e, date })
            })
            .collect();

        entries.sort_by_key(|entry| entry.date.sort_key());
        Self { entries }
    }

    /// Return all timeline entries in chronological order.
    pub fn entries(&self) -> &[TimelineEntry<'w>] {
        &self.entries
    }

    /// Filter entries to a year range (inclusive).
    pub fn range(self, from: Option<i64>, to: Option<i64>) -> Self {
        let entries = self
            .entries
            .into_iter()
            .filter(|entry| {
                if let Some(from_year) = from
                    && entry.date.year < from_year
                {
                    return false;
                }
                if let Some(to_year) = to
                    && entry.date.year > to_year
                {
                    return false;
                }
                true
            })
            .collect();
        Self { entries }
    }

    /// Filter to entries involving a specific entity (via relationships).
    pub fn involving(self, world: &'w World, entity_id: EntityId) -> Self {
        let entries = self
            .entries
            .into_iter()
            .filter(|entry| {
                // Check if the event entity is directly connected to the target entity
                world
                    .neighbors(entry.entity.id)
                    .iter()
                    .any(|(id, _)| *id == entity_id)
            })
            .collect();
        Self { entries }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{EventComponent, WorldDate};
    use crate::relationship::{Relationship, RelationshipKind};
    use crate::world::WorldMeta;

    fn world_with_events() -> World {
        let mut world = World::new(WorldMeta::new("Test"));

        let mut e1 = Entity::new(EntityKind::Event, "The Great Sundering");
        e1.components.event = Some(EventComponent {
            event_type: Some("cataclysm".to_string()),
            date: Some(WorldDate {
                year: -1247,
                month: Some(3),
                day: Some(15),
                era: None,
            }),
            duration: None,
            outcome: None,
        });
        world.add_entity(e1).unwrap();

        let mut e2 = Entity::new(EntityKind::Event, "The Founding of the Order");
        e2.components.event = Some(EventComponent {
            event_type: Some("political".to_string()),
            date: Some(WorldDate {
                year: -500,
                month: Some(1),
                day: None,
                era: None,
            }),
            duration: None,
            outcome: None,
        });
        world.add_entity(e2).unwrap();

        let mut e3 = Entity::new(EntityKind::Event, "The Battle of Ashfields");
        e3.components.event = Some(EventComponent {
            event_type: Some("battle".to_string()),
            date: Some(WorldDate {
                year: 12,
                month: Some(7),
                day: Some(1),
                era: None,
            }),
            duration: None,
            outcome: Some("pyrrhic victory".to_string()),
        });
        world.add_entity(e3).unwrap();

        // An event with no date (should be excluded from timeline)
        let mut e4 = Entity::new(EntityKind::Event, "The Prophecy");
        e4.components.event = Some(EventComponent {
            event_type: Some("prophecy".to_string()),
            date: None,
            duration: None,
            outcome: None,
        });
        world.add_entity(e4).unwrap();

        // A non-event entity (should be excluded)
        world
            .add_entity(Entity::new(EntityKind::Character, "Kael"))
            .unwrap();

        world
    }

    #[test]
    fn timeline_chronological_order() {
        let world = world_with_events();
        let tl = Timeline::from_world(&world);

        assert_eq!(tl.len(), 3);
        assert_eq!(tl.entries()[0].entity.name, "The Great Sundering");
        assert_eq!(tl.entries()[1].entity.name, "The Founding of the Order");
        assert_eq!(tl.entries()[2].entity.name, "The Battle of Ashfields");
    }

    #[test]
    fn timeline_range_filter() {
        let world = world_with_events();
        let tl = Timeline::from_world(&world).range(Some(-600), Some(0));

        assert_eq!(tl.len(), 1);
        assert_eq!(tl.entries()[0].entity.name, "The Founding of the Order");
    }

    #[test]
    fn timeline_involving_entity() {
        let mut world = world_with_events();

        let kael_id = world.find_id_by_name("Kael").unwrap();
        let battle_id = world.find_id_by_name("The Battle of Ashfields").unwrap();

        world
            .add_relationship(Relationship::new(
                battle_id,
                RelationshipKind::ParticipatedIn,
                kael_id,
            ))
            .unwrap();

        let tl = Timeline::from_world(&world).involving(&world, kael_id);
        assert_eq!(tl.len(), 1);
        assert_eq!(tl.entries()[0].entity.name, "The Battle of Ashfields");
    }

    #[test]
    fn timeline_empty_world() {
        let world = World::new(WorldMeta::new("Empty"));
        let tl = Timeline::from_world(&world);
        assert!(tl.is_empty());
    }
}
