use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{Jiffies, MessagePtr, Rank, TimerId};

pub(crate) enum Step {
    NetworkStep {
        from: Rank,
        to: Rank,
        message: MessagePtr,
    },
    TimerStep {
        to: Rank,
        id: TimerId,
    },
}

pub(crate) struct TimedStep {
    pub(crate) invocation_time: Jiffies,
    pub(crate) step: Step,
}

impl PartialEq for TimedStep {
    fn eq(&self, other: &Self) -> bool {
        self.invocation_time.eq(&other.invocation_time)
    }
}

impl Eq for TimedStep {}

impl PartialOrd for TimedStep {
    fn ge(&self, other: &Self) -> bool {
        self.invocation_time.ge(&other.invocation_time)
    }
    fn le(&self, other: &Self) -> bool {
        self.invocation_time.le(&other.invocation_time)
    }
    fn gt(&self, other: &Self) -> bool {
        self.invocation_time.gt(&other.invocation_time)
    }
    fn lt(&self, other: &Self) -> bool {
        self.invocation_time.lt(&other.invocation_time)
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.invocation_time.partial_cmp(&other.invocation_time)
    }
}

impl Ord for TimedStep {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.invocation_time.cmp(&other.invocation_time)
    }
}

pub(crate) type StepQueue = BinaryHeap<Reverse<TimedStep>>;
