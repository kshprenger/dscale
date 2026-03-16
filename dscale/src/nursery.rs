use std::sync::Arc;

use log::debug;

use crate::{
    Rank, event::DScaleMessage, global::set_process, process_handle::MutableProcessHandle,
};

pub(crate) type HandlerMap = Vec<MutableProcessHandle>;

pub(crate) struct Nursery {
    procs: HandlerMap,
}

impl Nursery {
    pub(crate) fn new(procs: HandlerMap) -> Arc<Self> {
        Arc::new(Self { procs })
    }

    pub(crate) fn start_single(&self, id: Rank) {
        set_process(id);
        debug!("Starting P{id}");
        self.procs
            .get(id)
            .expect("Invalid Rank")
            .lock()
            .expect("Process lock poisoned")
            .start();
    }

    pub(crate) fn deliver(&self, from: Rank, to: Rank, m: DScaleMessage) {
        let mut handle = self.procs.get(to).expect("Invalid Rank").lock().expect("Process lock poisoned");
        set_process(to);
        debug!("Delivering P{from} -> P{to}");
        match m {
            DScaleMessage::NetworkMessage(ptr) => handle.on_message(from, ptr),
            DScaleMessage::Timer(id) => handle.on_timer(id),
        }
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = Rank> {
        0..self.procs.len()
    }

    pub(crate) fn size(&self) -> usize {
        self.procs.len()
    }
}
