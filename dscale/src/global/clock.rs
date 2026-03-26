use std::sync::atomic::{AtomicUsize, Ordering};

use log::debug;

use crate::Jiffies;

pub(crate) static CLOCK: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn fast_forward_clock(future: Jiffies) {
    let present = Jiffies(CLOCK.swap(future.0, Ordering::Release));
    debug_assert!(present <= future, "Future < Present");
    debug!("Global time now: {future}");
}

/// Returns the current simulation time.
pub fn now() -> Jiffies {
    Jiffies(CLOCK.load(Ordering::Acquire))
}

pub(crate) fn reset() {
    CLOCK.store(0, Ordering::Release);
}
