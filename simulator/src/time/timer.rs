use std::collections::{BinaryHeap, HashMap};

use crate::{
    ProcessId, access,
    actor::SimulationActor,
    process::SharedProcessHandle,
    time::{Jiffies, Now},
    tso::NextGlobalUniqueId,
};

pub type TimerId = usize;

pub(crate) fn NextTimerId() -> TimerId {
    NextGlobalUniqueId()
}

// We cannot cancel timers yet. So user tracks them using TimerId
pub(crate) struct Timers {
    working_timers: BinaryHeap<(Jiffies, (ProcessId, TimerId))>,
    procs: HashMap<ProcessId, SharedProcessHandle>,
}

impl Timers {
    pub(crate) fn New(procs: HashMap<ProcessId, SharedProcessHandle>) -> Self {
        Self {
            working_timers: BinaryHeap::new(),
            procs,
        }
    }

    pub(crate) fn ScheduleTimers(&mut self, timers: &mut Vec<(ProcessId, TimerId, Jiffies)>) {
        timers
            .drain(..)
            .into_iter()
            .for_each(|(source, timer_id, after)| {
                self.working_timers
                    .push((Now() + after, (source, timer_id)));
            });
    }
}

impl SimulationActor for Timers {
    fn Start(&mut self) {
        // Do nothing
    }

    fn PeekClosest(&self) -> Option<Jiffies> {
        self.working_timers.peek().map(|(after, (_, _))| *after)
    }

    fn Step(&mut self) {
        let (_, (process_id, timer_id)) = self.working_timers.pop().expect("Should not be empty");
        access::SetProcess(process_id);
        self.procs
            .get_mut(&process_id)
            .expect("Invalid ProcessId")
            .borrow_mut()
            .OnTimer(timer_id);
    }
}
