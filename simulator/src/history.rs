use crate::{communication::Message, process::ProcessId};

/// (ProcessId, Event, ProcessId) <=> (Source, Event, Destination)
pub(crate) type ProcessStep<M: Message> = (ProcessId, M, ProcessId);
