use crate::ProcessId;

pub enum Destination {
    Broadcast,
    To(ProcessId),
}
