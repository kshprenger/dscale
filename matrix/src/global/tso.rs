use std::cell::Cell;

thread_local! {
    pub(crate) static TSO: Cell<usize> = Cell::new(0)
}

pub fn GlobalUniqueId() -> usize {
    TSO.with(|cell| {
        let result = cell.get();
        cell.set(result + 1);
        result
    })
}

pub(crate) fn Reset() {
    TSO.with(|cell| cell.set(0));
}
