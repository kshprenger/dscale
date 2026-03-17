use std::sync::{Arc, OnceLock};

use crossbeam_channel::Sender;
use crossbeam_utils::CachePadded;
use smallvec::SmallVec;

use crate::{Rank, event::Event, topology::Topology};

const PREDICTION_SCHEDULED_PER_STEP: usize = 2;

pub(crate) type EventBatch = SmallVec<[Event; PREDICTION_SCHEDULED_PER_STEP]>;

pub(crate) static SHARED_ACCESS: OnceLock<SharedAccess> = OnceLock::new();

pub(crate) fn setup_shared_access(topology: Arc<Topology>, sender: Sender<EventBatch>) {
    SHARED_ACCESS
        .set(SharedAccess::new(topology, sender))
        .expect("SharedAccess already initialized");
}

fn shared() -> &'static SharedAccess {
    SHARED_ACCESS.get().expect("SharedAccess not initialized")
}

#[derive(Debug)]
pub struct SharedAccess {
    topology: CachePadded<Arc<Topology>>,
    sender: CachePadded<Sender<EventBatch>>,
}

impl SharedAccess {
    pub(crate) fn new(topology: Arc<Topology>, sender: Sender<EventBatch>) -> Self {
        Self {
            topology: CachePadded::new(topology),
            sender: CachePadded::new(sender),
        }
    }

    pub(crate) fn sender(&self) -> &Sender<EventBatch> {
        &self.sender
    }

    pub(crate) fn topology(&self) -> &Topology {
        &self.topology
    }
}

pub(crate) fn sender() -> &'static Sender<EventBatch> {
    shared().sender()
}

pub(crate) fn topology() -> &'static Topology {
    shared().topology()
}

pub fn list_pool(pool_name: &str) -> &'static [Rank] {
    shared().topology.list_pool(pool_name)
}
