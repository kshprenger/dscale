use crate::{communication::Event, process::ProcessId};

pub(crate) type ProcessStep = (ProcessId, Event);
pub(crate) type ExecutionHistory = Vec<ProcessStep>;
