use std::cmp::Reverse;

use crate::{process::ProcessId, time::Jiffies};

pub trait Message: Eq + PartialEq + Ord + PartialOrd + Clone {
    fn VirtualSize(&self) -> usize;
}

// (Arrival time, source, dest, message)
pub type RoutedMessage<M> = (Jiffies, (ProcessId, ProcessId, M));

pub type TimePriorityMessageQueue<M> = std::collections::BinaryHeap<Reverse<RoutedMessage<M>>>;
