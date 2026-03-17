use std::cell::RefCell;
use std::sync::Arc;

use crate::destination::Destination;
use crate::event::Event;
use crate::global::shared_access::EventBatch;
use crate::random::{Randomizer, Seed};
use crate::{MessagePtr, global_unique_id, now};

use crate::{
    Message, Rank, debug_process,
    time::{Jiffies, timer_manager::TimerId},
    topology::GLOBAL_POOL,
};

thread_local! {
    pub(crate) static LOCAL_ACCESS: RefCell<LocalAccess> = RefCell::new(LocalAccess::default());
}

fn with_local_access<R>(f: impl FnOnce(&mut LocalAccess) -> R) -> R {
    LOCAL_ACCESS.with(|cell| f(&mut cell.borrow_mut()))
}

pub(crate) fn setup_local_access(seed: Seed) {
    with_local_access(|access| access.random = Randomizer::new(seed));
}

#[derive(Default)]
pub struct LocalAccess {
    process_on_execution: Rank,
    random: Randomizer,
    scheduled_events: EventBatch,
}

impl LocalAccess {
    fn broadcast_within_pool(&mut self, pool_name: &'static str, message: impl Message + 'static) {
        self.scheduled_events.push(Event::NetworkEvent {
            from: self.process_on_execution,
            to: Destination::BroadcastWithinPool(pool_name),
            message: MessagePtr(Arc::new(message)),
        });
    }

    fn send_to(&mut self, to: Rank, message: impl Message + 'static) {
        self.scheduled_events.push(Event::NetworkEvent {
            from: self.process_on_execution,
            to: Destination::To(to),
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
            to: rank(),
            id: timer_id,
            fire_after: after,
        });
        timer_id
    }

    fn set_process(&mut self, id: Rank) {
        self.process_on_execution = id
    }

    fn rank(&self) -> Rank {
        self.process_on_execution
    }
}

pub(crate) fn set_process(id: Rank) {
    with_local_access(|access| access.set_process(id));
}

pub fn schedule_timer_after(after: Jiffies) -> TimerId {
    debug_process!("Access: scheduling timer after {after}");
    with_local_access(|access| access.schedule_timer_after(after))
}

pub fn broadcast(message: impl Message + 'static) {
    debug_process!("Access: broadcasting globally");
    with_local_access(|access| access.broadcast_within_pool(GLOBAL_POOL, message));
}

pub fn broadcast_within_pool(pool: &'static str, message: impl Message + 'static) {
    debug_process!("Access: broadcasting within: {pool}");
    with_local_access(|access| access.broadcast_within_pool(pool, message));
}

pub fn send_to(to: Rank, message: impl Message + 'static) {
    debug_process!("Access: send to: P{to}");
    with_local_access(|access| access.send_to(to, message));
}

pub fn send_random(message: impl Message + 'static) {
    debug_process!("Access: sending random in GLOBAL_POOL");
    with_local_access(|access| access.send_random_from_pool(GLOBAL_POOL, message));
}

pub fn send_random_from_pool(pool: &'static str, message: impl Message + 'static) {
    debug_process!("Access: sending random from pool: {pool}");
    with_local_access(|access| access.send_random_from_pool(pool, message));
}

pub fn rank() -> Rank {
    with_local_access(|access| access.rank())
}

pub fn choose_from_pool(pool_name: &str) -> Rank {
    with_local_access(|access| access.choose_from_pool(pool_name))
}
