use crate::{Jiffies, MessagePtr, Rank, TimerId, destination::Destination};

#[derive(Clone)]
pub(crate) enum Event {
    NetworkEvent {
        from: Rank,
        to: Destination,
        message: MessagePtr,
    },
    TimerEvent {
        to: Rank,
        id: TimerId,
        fire_after: Jiffies,
    },
}
