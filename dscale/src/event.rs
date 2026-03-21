use crate::{Jiffies, MessagePtr, Rank, TimerId, destination::Destination};

#[derive(Clone)]
pub(crate) enum Event {
    NetworkEvent {
        source: Rank,
        destination: Destination,
        message: MessagePtr,
    },
    TimerEvent {
        rank: Rank,
        id: TimerId,
        fire_after: Jiffies,
    },
}
