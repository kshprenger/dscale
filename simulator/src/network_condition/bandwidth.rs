use std::collections::BinaryHeap;

use log::debug;

use crate::{
    communication::{Message, RoutedMessage, TimePriorityMessageQueue},
    network_condition::LatencyQueue,
    time::Jiffies,
};

#[derive(Clone, Copy)]
pub enum BandwidthType {
    Unbounded,
    Bounded(usize), // Bytes per Jiffy
}

#[derive(Clone, Copy)]
pub(crate) enum BandwidthQueueOptions<M: Message> {
    MessageArrivedByLatency,
    None,
    Some(RoutedMessage<M>),
}

pub(crate) struct BandwidthQueue<M: Message> {
    bandwidth: usize,
    global_queue: LatencyQueue<M>,
    current_buffers_sizes: Vec<usize>,
    merged_fifo_buffers: TimePriorityMessageQueue<M>,
}

impl<M: Message> BandwidthQueue<M> {
    pub(crate) fn new(
        bandwidth_type: BandwidthType,
        proc_num: usize,
        global_queue: LatencyQueue<M>,
    ) -> Self {
        let bandwidth = match bandwidth_type {
            BandwidthType::Unbounded => usize::MAX,
            BandwidthType::Bounded(bound) => bound,
        };

        Self {
            bandwidth,
            global_queue,
            current_buffers_sizes: vec![0; proc_num + 1],
            merged_fifo_buffers: BinaryHeap::new(),
        }
    }

    pub(crate) fn push(&mut self, message: RoutedMessage<M>) {
        debug!("Submitted message with base time: {}", message.0);
        self.global_queue.push(message);
    }

    pub(crate) fn pop(&mut self) -> BandwidthQueueOptions<M> {
        let closest_arriving_message = self.global_queue.peek();
        let closest_squeezing_message = self.merged_fifo_buffers.peek();

        match (closest_arriving_message, closest_squeezing_message) {
            (None, None) => BandwidthQueueOptions::None,
            (Some(_), None) => self.deliver_from_latency_queue(),
            (None, Some(_)) => self.deliver_from_buffer(),
            (Some(l_message), Some(b_message)) => {
                if l_message.0 <= b_message.0.0 {
                    self.deliver_from_latency_queue()
                } else {
                    self.deliver_from_buffer()
                }
            }
        }
    }
}

impl<M: Message> BandwidthQueue<M> {
    fn move_message_from_latency_queue_to_buffers(&mut self) {
        debug!("Moving message from latency queue to buffers");
        let mut message = self
            .global_queue
            .pop()
            .expect("Global queue should not be empty");
        self.current_buffers_sizes[message.1.0] += message.1.2.virtual_size();
        debug!(
            "New process {} buffer's size: {}",
            message.1.0, self.current_buffers_sizes[message.1.0]
        );
        debug!(
            "Message arrival time before bandwidth adjustment: {}",
            message.0
        );
        message.0 += Jiffies(self.current_buffers_sizes[message.1.0] / self.bandwidth);
        debug!(
            "Message arrival time after bandwidth adjustment: {}",
            message.0
        );
        self.merged_fifo_buffers.push(std::cmp::Reverse(message));
    }

    fn deliver_from_buffer(&mut self) -> BandwidthQueueOptions<M> {
        let message = self
            .merged_fifo_buffers
            .pop()
            .expect("All buffers should not be empty")
            .0;
        self.current_buffers_sizes[message.1.0] -= message.1.2.virtual_size();
        debug!(
            "New process {} buffer's size: {}",
            message.1.0, self.current_buffers_sizes[message.1.0]
        );
        BandwidthQueueOptions::Some(message)
    }

    fn deliver_from_latency_queue(&mut self) -> BandwidthQueueOptions<M> {
        self.move_message_from_latency_queue_to_buffers();
        BandwidthQueueOptions::MessageArrivedByLatency
    }
}
