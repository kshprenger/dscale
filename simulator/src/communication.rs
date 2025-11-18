use crate::{process::ProcessId, time::Jiffies};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum Event {
    Timeout(Jiffies),
    Message(Message),
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct Message {
    source: ProcessId,
    payload: bytes::Bytes,
}

pub type EventQueue = std::collections::BinaryHeap<(Jiffies, Event)>;
