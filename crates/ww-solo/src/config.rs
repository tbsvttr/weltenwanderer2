//! Configuration for a solo TTRPG session.

/// Configuration for a solo session.
#[derive(Debug, Clone)]
pub struct SoloConfig {
    /// RNG seed for reproducible oracle rolls.
    pub seed: u64,
    /// Initial chaos factor (1-9).
    pub initial_chaos: u32,
}

impl Default for SoloConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            initial_chaos: 5,
        }
    }
}

impl SoloConfig {
    /// Set the RNG seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set the initial chaos factor (clamped to 1-9).
    pub fn with_chaos(mut self, chaos: u32) -> Self {
        self.initial_chaos = chaos.clamp(1, 9);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = SoloConfig::default();
        assert_eq!(cfg.seed, 42);
        assert_eq!(cfg.initial_chaos, 5);
    }

    #[test]
    fn builder_methods() {
        let cfg = SoloConfig::default().with_seed(123).with_chaos(8);
        assert_eq!(cfg.seed, 123);
        assert_eq!(cfg.initial_chaos, 8);
    }

    #[test]
    fn chaos_clamped() {
        let cfg = SoloConfig::default().with_chaos(0);
        assert_eq!(cfg.initial_chaos, 1);
        let cfg = SoloConfig::default().with_chaos(99);
        assert_eq!(cfg.initial_chaos, 9);
    }
}
