use ww_core::entity::EntityId;

/// What kind of simulation event occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimEventKind {
    // Needs
    /// A need dropped below the critical threshold.
    NeedCritical {
        /// The entity whose need became critical.
        entity: EntityId,
        /// The name of the critical need.
        need: String,
    },
    /// A need was restored above the critical threshold.
    NeedSatisfied {
        /// The entity whose need was satisfied.
        entity: EntityId,
        /// The name of the satisfied need.
        need: String,
    },
    /// A need reached zero.
    NeedDepleted {
        /// The entity whose need was depleted.
        entity: EntityId,
        /// The name of the depleted need.
        need: String,
    },

    // Schedule
    /// An entity switched to a different scheduled activity.
    ActivityChanged {
        /// The entity that changed activity.
        entity: EntityId,
        /// The previous activity name.
        from: String,
        /// The new activity name.
        to: String,
    },

    // Spatial
    /// An entity left a location.
    Departed {
        /// The entity that departed.
        entity: EntityId,
        /// The location the entity left.
        from: EntityId,
    },
    /// An entity arrived at a location.
    Arrived {
        /// The entity that arrived.
        entity: EntityId,
        /// The location the entity arrived at.
        at: EntityId,
    },

    // Lifecycle
    /// An entity died.
    EntityDied {
        /// The entity that died.
        entity: EntityId,
        /// The cause of death.
        cause: String,
    },

    // Custom
    /// A user-defined event.
    Custom {
        /// A label identifying the custom event type.
        label: String,
        /// The entities involved in this custom event.
        entities: Vec<EntityId>,
    },
}

impl SimEventKind {
    /// Check whether a given entity is involved in this event.
    pub fn involves(&self, id: EntityId) -> bool {
        match self {
            Self::NeedCritical { entity, .. }
            | Self::NeedSatisfied { entity, .. }
            | Self::NeedDepleted { entity, .. }
            | Self::ActivityChanged { entity, .. }
            | Self::EntityDied { entity, .. } => *entity == id,
            Self::Departed { entity, from } => *entity == id || *from == id,
            Self::Arrived { entity, at } => *entity == id || *at == id,
            Self::Custom { entities, .. } => entities.contains(&id),
        }
    }
}

/// A record of something that happened during simulation.
#[derive(Debug, Clone)]
pub struct SimEvent {
    /// The simulation tick when this event occurred.
    pub tick: u64,
    /// The specific kind of event that occurred.
    pub kind: SimEventKind,
    /// A human-readable description of the event.
    pub description: String,
}

impl SimEvent {
    /// Create a new simulation event with the given tick, kind, and description.
    pub fn new(tick: u64, kind: SimEventKind, description: impl Into<String>) -> Self {
        Self {
            tick,
            kind,
            description: description.into(),
        }
    }
}

/// Accumulates events during a simulation run.
#[derive(Debug, Default)]
pub struct EventLog {
    events: Vec<SimEvent>,
    max_events: usize,
}

impl EventLog {
    /// Create a new event log with the given maximum capacity (0 = unlimited).
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    /// Append an event, dropping the oldest events if the log exceeds its capacity.
    pub fn push(&mut self, event: SimEvent) {
        self.events.push(event);
        if self.max_events > 0 && self.events.len() > self.max_events {
            let drain_count = self.events.len() - self.max_events;
            self.events.drain(..drain_count);
        }
    }

    /// Return a slice of all recorded events.
    pub fn events(&self) -> &[SimEvent] {
        &self.events
    }

    /// Return all events that occurred at the given tick.
    pub fn events_at_tick(&self, tick: u64) -> Vec<&SimEvent> {
        self.events.iter().filter(|e| e.tick == tick).collect()
    }

    /// Return all events involving the given entity.
    pub fn events_for_entity(&self, id: EntityId) -> Vec<&SimEvent> {
        self.events.iter().filter(|e| e.kind.involves(id)).collect()
    }

    /// Return the number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Return `true` if no events have been recorded.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Remove all recorded events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_log_push_and_query() {
        let mut log = EventLog::new(0);
        let id = EntityId::new();
        log.push(SimEvent::new(
            1,
            SimEventKind::NeedCritical {
                entity: id,
                need: "hunger".into(),
            },
            "test",
        ));
        assert_eq!(log.len(), 1);
        assert_eq!(log.events_at_tick(1).len(), 1);
        assert_eq!(log.events_for_entity(id).len(), 1);
    }

    #[test]
    fn event_log_max_events_trims() {
        let mut log = EventLog::new(2);
        let id = EntityId::new();
        for i in 0..5 {
            log.push(SimEvent::new(
                i,
                SimEventKind::NeedCritical {
                    entity: id,
                    need: "hunger".into(),
                },
                "test",
            ));
        }
        assert_eq!(log.len(), 2);
        // Oldest events were dropped, newest remain
        assert_eq!(log.events()[0].tick, 3);
        assert_eq!(log.events()[1].tick, 4);
    }

    #[test]
    fn event_kind_involves_entity() {
        let e1 = EntityId::new();
        let e2 = EntityId::new();
        let e3 = EntityId::new();

        // NeedCritical involves only its entity
        let kind = SimEventKind::NeedCritical {
            entity: e1,
            need: "hunger".into(),
        };
        assert!(kind.involves(e1));
        assert!(!kind.involves(e2));

        // Departed involves both entity and location
        let kind = SimEventKind::Departed {
            entity: e1,
            from: e2,
        };
        assert!(kind.involves(e1));
        assert!(kind.involves(e2));
        assert!(!kind.involves(e3));

        // Arrived involves both entity and location
        let kind = SimEventKind::Arrived { entity: e1, at: e2 };
        assert!(kind.involves(e1));
        assert!(kind.involves(e2));

        // Custom involves all listed entities
        let kind = SimEventKind::Custom {
            label: "test".into(),
            entities: vec![e1, e2],
        };
        assert!(kind.involves(e1));
        assert!(kind.involves(e2));
        assert!(!kind.involves(e3));
    }

    #[test]
    fn event_log_clear() {
        let mut log = EventLog::new(0);
        let id = EntityId::new();
        log.push(SimEvent::new(
            1,
            SimEventKind::NeedCritical {
                entity: id,
                need: "hunger".into(),
            },
            "test",
        ));
        assert!(!log.is_empty());
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn event_log_empty_queries() {
        let log = EventLog::new(0);
        let id = EntityId::new();
        assert!(log.events().is_empty());
        assert!(log.events_at_tick(0).is_empty());
        assert!(log.events_for_entity(id).is_empty());
        assert!(log.is_empty());
    }

    #[test]
    fn event_log_unlimited_capacity() {
        let mut log = EventLog::new(0);
        let id = EntityId::new();
        for i in 0..1000 {
            log.push(SimEvent::new(
                i,
                SimEventKind::NeedSatisfied {
                    entity: id,
                    need: "rest".into(),
                },
                "test",
            ));
        }
        // With max_events=0 (unlimited), all events retained
        assert_eq!(log.len(), 1000);
    }

    #[test]
    fn event_log_multi_tick_filtering() {
        let mut log = EventLog::new(0);
        let e1 = EntityId::new();
        let e2 = EntityId::new();
        // Push events across different ticks and entities
        log.push(SimEvent::new(
            1,
            SimEventKind::NeedCritical {
                entity: e1,
                need: "hunger".into(),
            },
            "e1 critical",
        ));
        log.push(SimEvent::new(
            1,
            SimEventKind::ActivityChanged {
                entity: e2,
                from: "idle".into(),
                to: "work".into(),
            },
            "e2 works",
        ));
        log.push(SimEvent::new(
            2,
            SimEventKind::NeedSatisfied {
                entity: e1,
                need: "hunger".into(),
            },
            "e1 satisfied",
        ));

        assert_eq!(log.events_at_tick(1).len(), 2);
        assert_eq!(log.events_at_tick(2).len(), 1);
        assert_eq!(log.events_at_tick(3).len(), 0);
        assert_eq!(log.events_for_entity(e1).len(), 2);
        assert_eq!(log.events_for_entity(e2).len(), 1);
    }
}
