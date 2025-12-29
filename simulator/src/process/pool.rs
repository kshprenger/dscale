use std::{
    cell::{RefCell, RefMut},
    collections::{BTreeMap, btree_map::Keys},
    rc::Rc,
};

use crate::{
    ProcessId,
    process::{MutableProcessHandle, handle::UniqueProcessHandle},
};

pub(crate) struct ProcessPool {
    // btree for deterministic iterators
    procs: BTreeMap<ProcessId, MutableProcessHandle>,
}

impl ProcessPool {
    pub(crate) fn NewShared(procs: Vec<(ProcessId, UniqueProcessHandle)>) -> Rc<Self> {
        Rc::new(Self {
            procs: procs
                .into_iter()
                .map(|(k, v)| (k, RefCell::new(v)))
                .collect(),
        })
    }

    pub(crate) fn Get(&self, id: ProcessId) -> RefMut<'_, UniqueProcessHandle> {
        self.procs.get(&id).expect("Invalid ProcessId").borrow_mut()
    }

    // Note: deterministic
    pub(crate) fn IterMut(
        &self,
    ) -> impl Iterator<Item = (&ProcessId, RefMut<'_, UniqueProcessHandle>)> {
        self.procs
            .iter()
            .map(|(id, handle)| (id, handle.borrow_mut()))
    }

    pub(crate) fn Keys(&self) -> Keys<'_, ProcessId, MutableProcessHandle> {
        self.procs.keys()
    }

    pub(crate) fn Size(&self) -> usize {
        self.procs.len()
    }
}
