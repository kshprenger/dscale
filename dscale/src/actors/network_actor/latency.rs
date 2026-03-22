use std::collections::BinaryHeap;
use std::sync::Arc;

use log::debug;

use crate::jiffy::Jiffies;
use crate::random::Randomizer;
use crate::step::{Step, StepQueue, TimedStep};
use crate::topology::Topology;

pub(crate) struct LatencyQueue {
    topology: Arc<Topology>,
    randomizer: Randomizer,
    queue: StepQueue,
}

impl LatencyQueue {
    pub(crate) fn new(randomizer: Randomizer, topology: Arc<Topology>) -> Self {
        Self {
            randomizer,
            topology,
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn push(&mut self, mut message: TimedStep) {
        debug!("Before latency: {}", message.invocation_time);
        let Step::NetworkStep { source, target, .. } = &message.step else {
            unreachable!("LatencyQueue only accepts NetworkSteps");
        };
        let distribution = self.topology.get_distribution(*source, *target);
        message.invocation_time += self.randomizer.random_usize(distribution);
        debug!("After latency: {}", message.invocation_time);
        self.queue.push(std::cmp::Reverse(message));
    }

    pub(crate) fn pop(&mut self) -> Option<TimedStep> {
        Some(self.queue.pop()?.0)
    }

    pub(crate) fn peek(&self) -> Option<Jiffies> {
        Some(self.queue.peek()?.0.invocation_time)
    }
}
