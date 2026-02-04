use ww_core::entity::EntityId;

/// What kind of simulation event occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimEventKind {
    // Needs
    NeedCritical {
        entity: EntityId,
        need: String,
    },
    NeedSatisfied {
        entity: EntityId,
        need: String,
    },
    NeedDepleted {
        entity: EntityId,
        need: String,
    },

    // Schedule
    ActivityChanged {
        entity: EntityId,
        from: String,
        to: String,
    },

    // Spatial
    Departed {
        entity: EntityId,
        from: EntityId,
    },
    Arrived {
        entity: EntityId,
        at: EntityId,
    },

    // Lifecycle
    EntityDied {
        entity: EntityId,
        cause: String,
    },

    // Custom
    Custom {
        label: String,
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
    pub tick: u64,
    pub kind: SimEventKind,
    pub description: String,
}

impl SimEvent {
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
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    pub fn push(&mut self, event: SimEvent) {
        self.events.push(event);
        if self.max_events > 0 && self.events.len() > self.max_events {
            let drain_count = self.events.len() - self.max_events;
            self.events.drain(..drain_count);
        }
    }

    pub fn events(&self) -> &[SimEvent] {
        &self.events
    }

    pub fn events_at_tick(&self, tick: u64) -> Vec<&SimEvent> {
        self.events.iter().filter(|e| e.tick == tick).collect()
    }

    pub fn events_for_entity(&self, id: EntityId) -> Vec<&SimEvent> {
        self.events.iter().filter(|e| e.kind.involves(id)).collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

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
}
