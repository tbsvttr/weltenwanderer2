use std::collections::HashMap;

use ww_core::component::CharacterStatus;
use ww_core::entity::{EntityId, EntityKind};

use crate::context::SimContext;
use crate::error::SimResult;
use crate::event::SimEventKind;
use crate::system::System;

/// Built-in need categories.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NeedKind {
    Hunger,
    Rest,
    Social,
    Safety,
    Custom(String),
}

impl std::fmt::Display for NeedKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hunger => write!(f, "hunger"),
            Self::Rest => write!(f, "rest"),
            Self::Social => write!(f, "social"),
            Self::Safety => write!(f, "safety"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// Current need levels for a single entity. Each value is 0.0..=1.0.
#[derive(Debug, Clone)]
pub struct NeedState {
    pub levels: HashMap<NeedKind, f64>,
}

impl NeedState {
    /// Create a new NeedState with all needs at 1.0 (fully satisfied).
    pub fn full(needs: &[NeedKind]) -> Self {
        let levels = needs.iter().map(|n| (n.clone(), 1.0)).collect();
        Self { levels }
    }

    pub fn get(&self, need: &NeedKind) -> Option<f64> {
        self.levels.get(need).copied()
    }

    /// Decrease a need by `amount`, clamping to 0.0.
    pub fn decay(&mut self, need: &NeedKind, amount: f64) {
        if let Some(level) = self.levels.get_mut(need) {
            *level = (*level - amount).max(0.0);
        }
    }

    /// Increase a need by `amount`, clamping to 1.0.
    pub fn satisfy(&mut self, need: &NeedKind, amount: f64) {
        if let Some(level) = self.levels.get_mut(need) {
            *level = (*level + amount).min(1.0);
        }
    }

    /// Returns needs that are at or below the critical threshold.
    pub fn critical_needs(&self, threshold: f64) -> Vec<&NeedKind> {
        self.levels
            .iter()
            .filter(|(_, v)| **v <= threshold)
            .map(|(k, _)| k)
            .collect()
    }
}

/// Configuration for the needs system.
#[derive(Debug, Clone)]
pub struct NeedsConfig {
    /// Which needs to track.
    pub needs: Vec<NeedKind>,
    /// Decay rate per tick for each need kind.
    pub decay_rates: HashMap<NeedKind, f64>,
    /// Threshold below which a need is considered critical.
    pub critical_threshold: f64,
    /// Threshold at which entity dies (only for lethal needs).
    pub death_threshold: f64,
    /// Which needs cause death when depleted.
    pub lethal_needs: Vec<NeedKind>,
}

impl Default for NeedsConfig {
    fn default() -> Self {
        let needs = vec![
            NeedKind::Hunger,
            NeedKind::Rest,
            NeedKind::Social,
            NeedKind::Safety,
        ];
        let mut decay_rates = HashMap::new();
        for need in &needs {
            decay_rates.insert(need.clone(), 0.01);
        }
        Self {
            needs,
            decay_rates,
            critical_threshold: 0.15,
            death_threshold: 0.0,
            lethal_needs: vec![NeedKind::Hunger],
        }
    }
}

/// Tracks and decays need levels for all Character entities.
#[derive(Debug)]
pub struct NeedsSystem {
    config: NeedsConfig,
    states: HashMap<EntityId, NeedState>,
}

impl NeedsSystem {
    pub fn new(config: NeedsConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(NeedsConfig::default())
    }

    pub fn get_state(&self, id: EntityId) -> Option<&NeedState> {
        self.states.get(&id)
    }

    pub fn get_state_mut(&mut self, id: EntityId) -> Option<&mut NeedState> {
        self.states.get_mut(&id)
    }

    pub fn all_states(&self) -> &HashMap<EntityId, NeedState> {
        &self.states
    }

    fn ensure_tracked(&mut self, id: EntityId) {
        self.states
            .entry(id)
            .or_insert_with(|| NeedState::full(&self.config.needs));
    }

    fn is_alive(ctx: &SimContext<'_>, id: EntityId) -> bool {
        ctx.world
            .get_entity(id)
            .and_then(|e| e.components.character.as_ref())
            .is_some_and(|c| c.status == CharacterStatus::Alive)
    }
}

impl System for NeedsSystem {
    fn name(&self) -> &str {
        "needs"
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
            self.ensure_tracked(id);
        }
        Ok(())
    }

    fn tick(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()> {
        let ids: Vec<EntityId> = self.states.keys().copied().collect();

        for id in ids {
            if !Self::is_alive(ctx, id) {
                continue;
            }

            for need in &self.config.needs.clone() {
                let rate = self.config.decay_rates.get(need).copied().unwrap_or(0.01);

                let state = match self.states.get_mut(&id) {
                    Some(s) => s,
                    None => continue,
                };

                let prev = state.get(need).unwrap_or(1.0);
                state.decay(need, rate);
                let curr = state.get(need).unwrap_or(0.0);

                // Check critical threshold crossing
                if prev > self.config.critical_threshold && curr <= self.config.critical_threshold {
                    ctx.emit(
                        SimEventKind::NeedCritical {
                            entity: id,
                            need: need.to_string(),
                        },
                        format!("{} has critical {}", ctx.world.entity_name(id), need),
                    );
                }

                // Check death threshold for lethal needs
                if curr <= self.config.death_threshold && self.config.lethal_needs.contains(need) {
                    if let Some(entity) = ctx.world.get_entity_mut(id)
                        && let Some(ref mut ch) = entity.components.character
                    {
                        ch.status = CharacterStatus::Dead;
                    }
                    ctx.emit(
                        SimEventKind::EntityDied {
                            entity: id,
                            cause: format!("{} depleted", need),
                        },
                        format!("{} died from {}", ctx.world.entity_name(id), need),
                    );
                }
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
    fn need_state_starts_full() {
        let state = NeedState::full(&[NeedKind::Hunger, NeedKind::Rest]);
        assert!((state.get(&NeedKind::Hunger).unwrap() - 1.0).abs() < f64::EPSILON);
        assert!((state.get(&NeedKind::Rest).unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn need_decay_clamps_at_zero() {
        let mut state = NeedState::full(&[NeedKind::Hunger]);
        state.decay(&NeedKind::Hunger, 2.0); // Exceed 1.0
        assert!((state.get(&NeedKind::Hunger).unwrap()).abs() < f64::EPSILON);
    }

    #[test]
    fn need_satisfy_clamps_at_one() {
        let mut state = NeedState::full(&[NeedKind::Hunger]);
        state.decay(&NeedKind::Hunger, 0.5);
        state.satisfy(&NeedKind::Hunger, 2.0); // Exceed remaining
        assert!((state.get(&NeedKind::Hunger).unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn critical_needs_detected() {
        let mut state = NeedState::full(&[NeedKind::Hunger, NeedKind::Rest]);
        state.decay(&NeedKind::Hunger, 0.9); // Now at 0.1
        let critical = state.critical_needs(0.15);
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0], &NeedKind::Hunger);
    }
}
