use crate::context::SimContext;
use crate::error::SimResult;

/// A simulation subsystem that runs each tick.
///
/// Systems are executed in registration order. Each system receives
/// a mutable context providing access to the world, clock, RNG, and
/// event log.
pub trait System: std::fmt::Debug {
    /// Human-readable name for this system.
    fn name(&self) -> &str;

    /// Called once per tick.
    fn tick(&mut self, ctx: &mut SimContext<'_>) -> SimResult<()>;

    /// Called once when the system is first registered. Optional setup hook.
    fn init(&mut self, _ctx: &mut SimContext<'_>) -> SimResult<()> {
        Ok(())
    }

    /// Support downcasting to concrete types for cross-system communication.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Support downcasting to concrete types for cross-system communication.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
