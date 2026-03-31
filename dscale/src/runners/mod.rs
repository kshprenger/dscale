mod progress;
pub(crate) mod scalable;
pub(crate) mod simple;
pub(super) mod task;
pub(crate) mod workers;

use crate::jiffy::Jiffies;

/// Outcome of a simulation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// Ran all requested steps (for `run_steps`) or the full sub-budget (for `run_sub_budget`).
    Completed { steps: usize },
    /// The total time budget was exhausted.
    BudgetExhausted { steps: usize },
    /// No more events to process — the simulation has quiesced.
    NoMoreEvents { steps: usize },
}

impl RunStatus {
    /// Number of steps that were actually executed.
    pub fn steps(&self) -> usize {
        match *self {
            RunStatus::Completed { steps }
            | RunStatus::BudgetExhausted { steps }
            | RunStatus::NoMoreEvents { steps } => steps,
        }
    }
}

/// Execution engine returned by [`SimulationBuilder::build`].
pub trait SimulationRunner {
    /// Runs the simulation until the total time budget is exhausted
    /// or no more events remain.
    fn run_full_budget(&mut self) -> RunStatus;

    /// Runs up to `k` steps and returns the outcome.
    ///
    /// The simulation can be resumed by calling this method again.
    fn run_steps(&mut self, k: usize) -> RunStatus;

    /// Runs the simulation for at most `sub_budget` additional time.
    ///
    /// Stops early if the total time budget is hit or no more events remain.
    /// The simulation can be resumed by calling any run method again.
    fn run_sub_budget(&mut self, sub_budget: Jiffies) -> RunStatus;
}
