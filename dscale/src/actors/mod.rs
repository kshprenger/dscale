pub(crate) mod network_actor;
pub(crate) mod timer_actor;

use crate::{
    event::Event,
    global::local_access::EventBatch,
    jiffy::Jiffies,
    step::Step,
};

use network_actor::NetworkActor;
use timer_actor::TimerActor;

pub(crate) trait SimulationActor {
    fn next_step(&mut self) -> Step;
    fn peek_next_step(&self) -> Option<Jiffies>;
    fn submit(&mut self, event: Event);
}

pub(crate) struct Actors {
    pub(crate) network: NetworkActor,
    pub(crate) timers: TimerActor,
}

impl Actors {
    pub(super) fn next_step(&mut self) -> Step {
        let t = self.timers.peek_next_step();
        let n = self.network.peek_next_step();
        match (t, n) {
            (Some(a), Some(b)) if a <= b => self.timers.next_step(),
            (Some(_), Some(_)) => self.network.next_step(),
            (Some(_), None) => self.timers.next_step(),
            (None, Some(_)) => self.network.next_step(),
            (None, None) => panic!("next_step called with no pending steps"),
        }
    }

    pub(super) fn peek_next_step(&self) -> Option<Jiffies> {
        let t = self.timers.peek_next_step();
        let n = self.network.peek_next_step();
        match (t, n) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    pub(super) fn submit(&mut self, events: &mut EventBatch) {
        for event in events.drain(..) {
            match event {
                e @ Event::TimerEvent { .. } => self.timers.submit(e),
                e @ Event::NetworkEvent { .. } => self.network.submit(e),
            }
        }
    }
}
