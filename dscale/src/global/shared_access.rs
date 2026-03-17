use std::sync::{Arc, OnceLock};

use crossbeam_utils::CachePadded;

use crate::{Rank, topology::Topology};

pub(crate) static SHARED_ACCESS: OnceLock<SharedAccess> = OnceLock::new();

pub(crate) fn setup_shared_access(topology: Arc<Topology>) {
    SHARED_ACCESS
        .set(SharedAccess::new(topology))
        .expect("SharedAccess already initialized");
}

fn shared() -> &'static SharedAccess {
    SHARED_ACCESS.get().expect("SharedAccess not initialized")
}

#[derive(Debug)]
pub struct SharedAccess {
    topology: CachePadded<Arc<Topology>>,
}

impl SharedAccess {
    pub(crate) fn new(topology: Arc<Topology>) -> Self {
        Self {
            topology: CachePadded::new(topology),
        }
    }

    pub(crate) fn topology(&self) -> &Topology {
        &self.topology
    }
}

pub(crate) fn topology() -> &'static Topology {
    shared().topology()
}

pub fn list_pool(pool_name: &str) -> &'static [Rank] {
    shared().topology.list_pool(pool_name)
}
