use std::cell::Cell;

use log::debug;

use crate::Jiffies;

thread_local! {
    pub(crate) static CLOCK: Cell<Jiffies> = Cell::new(Jiffies(0))
}

pub(crate) fn FastForwardClock(future: Jiffies) {
    CLOCK.with(|cell| {
        let present = cell.get();
        debug_assert!(present <= future, "Future < Present");
        cell.set(future);
        debug!("Global time now: {future}");
    });
}

pub fn Now() -> Jiffies {
    CLOCK.with(|cell| cell.get())
}
