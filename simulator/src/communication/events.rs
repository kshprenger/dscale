use priority_queue::PriorityQueue;

use crate::{process::ProcessId, time::Jiffies};

pub type EventId = usize;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub(crate) struct Event {
    pub id: EventId,
    pub event_type: EventType,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub(crate) enum EventType {
    Timeout,
    Message(Message),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub enum Destination {
    Broadcast,
    SendSelf,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct Message {
    source: ProcessId,
    payload: bytes::Bytes,
}

/// (Jiffies, Event) <=> At speciffied timestamp event will be delivered
pub type EventDeliveryQueue = PriorityQueue<Event, Jiffies>;
