use std::{collections::BTreeMap, rc::Rc};

use log::debug;

use crate::{
    Rank, dscale_message::DScaleMessage, global::set_process, process_handle::MutableProcessHandle,
};

pub(crate) type HandlerMap = BTreeMap<Rank, MutableProcessHandle>; // btree for deterministic iterators

pub(crate) struct Nursery {
    procs: HandlerMap,
}

impl Nursery {
    pub(crate) fn new(procs: HandlerMap) -> Rc<Self> {
        Rc::new(Self { procs })
    }

    pub(crate) fn start_single(&self, id: Rank) {
        set_process(id);
        debug!("Starting P{id}");
        self.procs
            .get(&id)
            .expect("Invalid Rank")
            .borrow_mut()
            .start();
    }

    pub(crate) fn deliver(&self, from: Rank, to: Rank, m: DScaleMessage) {
        let mut handle = self.procs.get(&to).expect("Invalid Rank").borrow_mut();
        set_process(to);
        debug!("Delivering P{from} -> P{to}");
        match m {
            DScaleMessage::NetworkMessage(ptr) => handle.on_message(from, ptr),
            DScaleMessage::Timer(id) => handle.on_timer(id),
        }
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &Rank> {
        self.procs.keys()
    }

    pub(crate) fn size(&self) -> usize {
        self.procs.len()
    }
}
