use crate::{history::ExecutionHistory, metrics::Metrics};

pub(crate) enum SimulationResult {
    Ok(Metrics),
    Deadlock(ExecutionHistory),
}
