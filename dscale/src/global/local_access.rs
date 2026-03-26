use std::cell::RefCell;
use std::mem;
use std::sync::Arc;

use crossbeam_channel::Sender;
use smallvec::SmallVec;

use crate::destination::Destination;
use crate::event::Event;
use crate::random::{Randomizer, Seed};
use crate::runners::task::{TaskId, TaskResult};
use crate::{MessagePtr, global_unique_id, now};

use crate::{
    Message, Rank, actors::timer_actor::TimerId, debug_process, jiffy::Jiffies,
    topology::GLOBAL_POOL,
};

const PREDICTION_SCHEDULED_PER_STEP: usize = 2;

pub(crate) type EventBatch = SmallVec<[Event; PREDICTION_SCHEDULED_PER_STEP]>;

thread_local! {
    pub(crate) static LOCAL_ACCESS: RefCell<LocalAccess> = RefCell::new(LocalAccess::default());
}

fn with_local_access<R>(f: impl FnOnce(&mut LocalAccess) -> R) -> R {
    LOCAL_ACCESS.with(|cell| f(&mut cell.borrow_mut()))
}

pub(crate) fn setup_local_access(seed: Seed, coordinator: Sender<TaskResult>) {
    with_local_access(|access| {
        access.random = Randomizer::new(seed);
        access.coordinator = Some(coordinator)
    });
}

#[derive(Default)]
pub(crate) struct LocalAccess {
    process_on_execution: Rank,
    current_task: TaskId,
    random: Randomizer,
    scheduled_events: EventBatch,
    coordinator: Option<Sender<TaskResult>>,
}

impl LocalAccess {
    fn broadcast_within_pool(&mut self, pool_name: &'static str, message: impl Message + 'static) {
        self.scheduled_events.push(Event::NetworkEvent {
            source: self.process_on_execution,
            destination: Destination::BroadcastWithinPool(pool_name),
            message: MessagePtr(Arc::new(message)),
        });
    }

    fn send_to(&mut self, rank: Rank, message: impl Message + 'static) {
        self.scheduled_events.push(Event::NetworkEvent {
            source: self.process_on_execution,
            destination: Destination::Target(rank),
            message: MessagePtr(Arc::new(message)),
        });
    }

    fn send_random_from_pool(&mut self, pool: &str, message: impl Message + 'static) {
        let target = self.choose_from_pool(pool);
        self.send_to(target, message);
    }

    fn choose_from_pool(&mut self, pool_name: &str) -> Rank {
        let pool = super::shared_access::list_pool(pool_name);
        self.random.choose_from_slice(pool)
    }

    fn schedule_timer_after(&mut self, after: Jiffies) -> TimerId {
        let timer_id = global_unique_id();
        self.scheduled_events.push(Event::TimerEvent {
            rank: self.process_on_execution,
            id: timer_id,
            fire_after: after,
        });
        timer_id
    }

    fn set_task(&mut self, task_id: TaskId, proc_id: Rank) {
        self.process_on_execution = proc_id;
        self.current_task = task_id;
    }

    fn done(&mut self) {
        let _ = self
            .coordinator
            .as_ref()
            .expect("No coordinator")
            .send(TaskResult {
                id: self.current_task,
                rank: self.process_on_execution,
                events: mem::take(&mut self.scheduled_events),
            });
    }

    fn take_events(&mut self) -> EventBatch {
        mem::take(&mut self.scheduled_events)
    }

    fn rank(&self) -> Rank {
        self.process_on_execution
    }
}

pub(crate) fn set_task(task_id: TaskId, proc_id: Rank) {
    with_local_access(|access| access.set_task(task_id, proc_id));
}

pub(crate) fn done() {
    with_local_access(|access| access.done());
}

pub(crate) fn take_events() -> EventBatch {
    with_local_access(|access| access.take_events())
}

/// Schedules a timer for the current process, firing after the given delay.
/// Returns a [`TimerId`] that will be passed to [`crate::ProcessHandle::on_timer`].
pub fn schedule_timer_after(after: Jiffies) -> TimerId {
    debug_process!("[Access] scheduling timer after {after}");
    with_local_access(|access| access.schedule_timer_after(after))
}

/// Sends a message to all processes in [`GLOBAL_POOL`] (i.e. every process).
pub fn broadcast(message: impl Message + 'static) {
    with_local_access(|access| access.broadcast_within_pool(GLOBAL_POOL, message));
}

/// Sends a message to all processes within the named pool.
pub fn broadcast_within_pool(pool: &'static str, message: impl Message + 'static) {
    debug_process!("[Access] broadcasting within: {pool}");
    with_local_access(|access| access.broadcast_within_pool(pool, message));
}

/// Sends a message to the process with the given rank.
pub fn send_to(rank: Rank, message: impl Message + 'static) {
    debug_process!("[Access] send to: P{rank}");
    with_local_access(|access| access.send_to(rank, message));
}

/// Sends a message to a randomly chosen process from [`GLOBAL_POOL`].
pub fn send_random(message: impl Message + 'static) {
    debug_process!("[Access] sending random P from GLOBAL_POOL");
    with_local_access(|access| access.send_random_from_pool(GLOBAL_POOL, message));
}

/// Sends a message to a randomly chosen process from the named pool.
pub fn send_random_from_pool(pool: &'static str, message: impl Message + 'static) {
    debug_process!("[Access] sending random from pool {pool}");
    with_local_access(|access| access.send_random_from_pool(pool, message));
}

/// Returns the rank of the currently executing process.
pub fn rank() -> Rank {
    with_local_access(|access| access.rank())
}

/// Picks a random process rank from the named pool.
pub fn choose_from_pool(pool_name: &str) -> Rank {
    with_local_access(|access| access.choose_from_pool(pool_name))
}

pub(crate) fn reset() {
    LOCAL_ACCESS.with(|cell| *cell.borrow_mut() = LocalAccess::default());
}
