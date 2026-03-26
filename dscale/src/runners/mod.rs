pub(crate) mod simple;
mod emojis;
mod progress;
pub(crate) mod scalable;
pub(super) mod task;
pub(crate) mod workers;

/// Execution engine returned by [`SimulationBuilder::build`].
pub trait SimulationRunner {
    /// Runs the simulation until the configured time budget is exhausted.
    fn run_full_budget(&mut self);
}
