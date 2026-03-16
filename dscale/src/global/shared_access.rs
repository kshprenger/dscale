use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crossbeam_channel::Sender;
use crossbeam_utils::CachePadded;
use smallvec::SmallVec;

use crate::{Rank, event::Event, random::Randomizer, topology::Topology};

const PREDICTION_SCHEDULED_PER_STEP: usize = 2;

pub(crate) type EventBatch = SmallVec<[Event; PREDICTION_SCHEDULED_PER_STEP]>;

pub(crate) static SHARED_ACCESS: OnceLock<RwLock<SharedAccess>> = OnceLock::new();

pub(crate) fn setup_shared_access(
    topology: Arc<Topology>,
    random: Randomizer,
    sender: Sender<EventBatch>,
) {
    SHARED_ACCESS
        .set(RwLock::new(SharedAccess::new(topology, random, sender)))
        .expect("SharedAccess already initialized");
}

#[derive(Debug)]
pub struct SharedAccess {
    topology: CachePadded<Arc<Topology>>,
    random: CachePadded<Randomizer>,
    sender: CachePadded<Sender<EventBatch>>,
}

impl SharedAccess {
    pub(crate) fn new(
        topology: Arc<Topology>,
        random: Randomizer,
        sender: Sender<EventBatch>,
    ) -> Self {
        Self {
            topology: CachePadded::new(topology),
            random: CachePadded::new(random),
            sender: CachePadded::new(sender),
        }
    }
}

impl SharedAccess {
    fn list_pool(&self, name: &str) -> &[Rank] {
        self.topology.list_pool(name)
    }

    fn choose_from_pool(&mut self, pool_name: &str) -> Rank {
        self.random
            .choose_from_slice(&self.topology.list_pool(pool_name))
    }
}

fn write() -> RwLockWriteGuard<'static, SharedAccess> {
    SHARED_ACCESS.get().unwrap().write().unwrap()
}

fn read() -> RwLockReadGuard<'static, SharedAccess> {
    SHARED_ACCESS.get().unwrap().read().unwrap()
}

pub(crate) fn choose_from_pool(pool_name: &str) -> Rank {
    write().choose_from_pool(pool_name)
}

pub fn list_pool(pool_name: &str) -> Vec<Rank> {
    read().list_pool(pool_name).to_vec()
}
