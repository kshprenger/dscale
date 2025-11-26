use std::cmp::Reverse;

use crate::{
    communication::{Message, TimePriorityMessageQueue},
    process::ProcessId,
    time::Jiffies,
};

#[derive(Clone, Copy)]
pub enum BandwidthType {
    Unbounded,
    Bounded(usize), // Bytes per Jiffy
}

pub(crate) struct NetworkBoundedMessageQueue<M: Message> {
    bandwidth: usize,
    total_passed: usize,
    queue: TimePriorityMessageQueue<M>,
}

impl<M: Message> NetworkBoundedMessageQueue<M> {
    pub(crate) fn new(bandwidth_type: BandwidthType) -> Self {
        let bandwidth = match bandwidth_type {
            BandwidthType::Unbounded => usize::MAX,
            BandwidthType::Bounded(bound) => bound,
        };

        Self {
            bandwidth,
            total_passed: 0,
            queue: TimePriorityMessageQueue::new(),
        }
    }

    pub(crate) fn push(&mut self, message: (ProcessId, M), should_arrive_at: Jiffies) {
        self.queue.push(Reverse((should_arrive_at, message)));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(crate) fn peek(&self) -> Option<&Reverse<(Jiffies, (ProcessId, M))>> {
        self.queue.peek()
    }

    pub(crate) fn try_pop(&mut self, current_time: Jiffies) -> Option<(Jiffies, (ProcessId, M))> {
        match self.queue.peek() {
            None => None,
            Some(Reverse((_, (_, message)))) => {
                if self.bandwidth == usize::MAX {
                    return Some(self.queue.pop().unwrap().0);
                }
                if self.total_passed + message.virtual_size() > self.bandwidth * current_time {
                    None
                } else {
                    self.total_passed += message.virtual_size();
                    Some(self.queue.pop().unwrap().0)
                }
            }
        }
    }
}
