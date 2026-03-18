
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) static TSO: AtomicUsize = AtomicUsize::new(0);

pub fn global_unique_id() -> usize {
    TSO.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn drop() {
    TSO.store(0, Ordering::Relaxed);
}
