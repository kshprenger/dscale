use std::sync::{
    Arc,
    atomic::{AtomicPtr, Ordering},
};

use crate::{Rank, topology::Topology};

static SHARED_ACCESS: AtomicPtr<SharedAccess> = AtomicPtr::new(std::ptr::null_mut());

pub(crate) fn setup_shared_access(topology: Arc<Topology>) {
    let sa = Box::new(SharedAccess::new(topology));
    let ptr = Box::into_raw(sa);
    let old = SHARED_ACCESS.swap(ptr, Ordering::Release);
    if !old.is_null() {
        // Drop previous allocation
        unsafe {
            drop(Box::from_raw(old));
        }
    }
}

fn shared() -> &'static SharedAccess {
    let ptr = SHARED_ACCESS.load(Ordering::Acquire);
    assert!(!ptr.is_null(), "SharedAccess not initialized");
    unsafe { &*ptr }
}

#[derive(Debug)]
struct SharedAccess {
    topology: Arc<Topology>,
}

impl SharedAccess {
    fn new(topology: Arc<Topology>) -> Self {
        Self { topology }
    }
}

/// Returns a slice of all process ranks in the named pool.
pub fn list_pool(pool_name: &str) -> &'static [Rank] {
    shared().topology.list_pool(pool_name)
}

pub(crate) fn reset() {
    let old = SHARED_ACCESS.swap(std::ptr::null_mut(), Ordering::Release);
    if !old.is_null() {
        unsafe {
            drop(Box::from_raw(old));
        }
    }
}
