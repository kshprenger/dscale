use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{Jiffies, global::local_access::EventBatch};

pub(crate) type TaskId = (Jiffies, usize);

// Sorting by arrival time, then by id
pub(super) type TaskIndex = BinaryHeap<Reverse<TaskId>>;

pub(crate) struct TaskResult {
    pub(crate) id: TaskId,
    pub(crate) rank: usize,
    pub(crate) events: EventBatch,
}
