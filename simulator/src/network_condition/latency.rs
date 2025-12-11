use std::collections::BinaryHeap;

use log::debug;

use crate::Message;
use crate::communication::{RoutedMessage, TimePriorityMessageQueue};
use crate::{random::Randomizer, time::Jiffies};

pub(crate) struct LatencyQueue<M: Message> {
    randomizer: Randomizer,
    max_latency: Jiffies,
    queue: TimePriorityMessageQueue<M>,
}
impl<M: Message> LatencyQueue<M> {
    pub(crate) fn New(randomizer: Randomizer, max_latency: Jiffies) -> Self {
        Self {
            randomizer,
            max_latency,
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn Push(&mut self, mut message: RoutedMessage<M>) {
        debug!("Arrival time before adding latency: {}", message.0);
        message.0 += self.randomizer.RandomFromRange(0, self.max_latency.0);
        debug!("Arrival time after adding random latency: {}", message.0);
        self.queue.push(std::cmp::Reverse(message));
    }

    pub(crate) fn Pop(&mut self) -> Option<RoutedMessage<M>> {
        Some(self.queue.pop()?.0)
    }

    pub(crate) fn Peek(&mut self) -> Option<&RoutedMessage<M>> {
        Some(&self.queue.peek()?.0)
    }
}
