use std::collections::BinaryHeap;

use crate::{
    actors::network_actor::LatencyQueue,
    jiffy::Jiffies,
    now,
    step::{Step, StepQueue, TimedStep},
};

#[derive(Clone, Copy, Default)]
pub enum BandwidthDescription {
    #[default]
    Unbounded,

    Bounded(usize), // Bytes per Jiffy
}

pub(crate) struct BandwidthQueue {
    bandwidth: usize,
    global_queue: LatencyQueue,
    total_pased: Vec<usize>,
    merged_fifo_buffers: StepQueue,
}

impl BandwidthQueue {
    pub(crate) fn new(
        bandwidth_type: BandwidthDescription,
        proc_num: usize,
        global_queue: LatencyQueue,
    ) -> Self {
        let bandwidth = match bandwidth_type {
            BandwidthDescription::Unbounded => usize::MAX,
            BandwidthDescription::Bounded(bound) => bound,
        };

        Self {
            bandwidth,
            global_queue,
            total_pased: vec![0; proc_num + 1],
            merged_fifo_buffers: BinaryHeap::new(),
        }
    }

    pub(crate) fn push(&mut self, message: TimedStep) {
        self.global_queue.push(message);
    }

    pub(crate) fn pop(&mut self) -> Option<TimedStep> {
        loop {
            let latency_time = self.global_queue.peek();
            let buffer_time = self.merged_fifo_buffers.peek().map(|e| e.0.invocation_time);

            match (latency_time, buffer_time) {
                (None, None) => return None,
                (Some(_), None) => {
                    if let Some(step) = self.deliver_from_latency_queue() {
                        return Some(step);
                    }
                    // Bounded: message moved to buffer, re-check.
                }
                (None, Some(_)) => return self.deliver_from_buffer(),
                (Some(lt), Some(bt)) => {
                    if lt <= bt {
                        if let Some(step) = self.deliver_from_latency_queue() {
                            return Some(step);
                        }
                        // Bounded: message moved to buffer, re-check.
                    } else {
                        return self.deliver_from_buffer();
                    }
                }
            }
        }
    }

    pub(crate) fn peek_closest(&self) -> Option<Jiffies> {
        let latency_time = self.global_queue.peek();
        let buffer_time = self.merged_fifo_buffers.peek().map(|e| e.0.invocation_time);

        match (latency_time, buffer_time) {
            (None, None) => None,
            (Some(t), None) | (None, Some(t)) => Some(t),
            (Some(lt), Some(bt)) => Some(lt.min(bt)),
        }
    }
}

impl BandwidthQueue {
    fn move_message_from_latency_queue_to_buffers(&mut self) {
        let mut message = self
            .global_queue
            .pop()
            .expect("Global queue should not be empty");

        let Step::NetworkStep {
            target,
            message: ref msg,
            ..
        } = message.step
        else {
            unreachable!("BandwidthQueue only accepts NetworkSteps");
        };
        let new_total = self.total_pased[target] + msg.0.virtual_size();

        if new_total > now().0 * self.bandwidth {
            message.invocation_time = Jiffies(new_total / self.bandwidth); // > now()
        }

        self.merged_fifo_buffers.push(std::cmp::Reverse(message));
    }

    fn deliver_from_buffer(&mut self) -> Option<TimedStep> {
        let timed_step = self
            .merged_fifo_buffers
            .pop()
            .expect("All buffers should not be empty")
            .0;
        let Step::NetworkStep {
            target,
            message: ref msg,
            ..
        } = timed_step.step
        else {
            unreachable!("BandwidthQueue only accepts NetworkSteps");
        };
        self.total_pased[target] += msg.0.virtual_size();
        Some(timed_step)
    }

    fn deliver_from_latency_queue(&mut self) -> Option<TimedStep> {
        if self.bandwidth == usize::MAX {
            // For unbounded bandwidth, deliver directly from latency queue
            // (Fast-Path)
            let message = self
                .global_queue
                .pop()
                .expect("Global queue should not be empty");
            Some(message)
        } else {
            // For bounded bandwidth, move to buffers first
            self.move_message_from_latency_queue_to_buffers();
            None
        }
    }
}
