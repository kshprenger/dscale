pub(crate) mod deterministic;
mod emojis;
mod progress;
pub(crate) mod scalable;
pub(super) mod task;

pub trait SimulationRunner {
    fn run_full_budget(&mut self);
}
