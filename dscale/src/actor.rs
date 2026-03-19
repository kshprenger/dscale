use crate::{
    event::Event,
    network::Network,
    step::Step,
    time::{Jiffies, timer_manager::TimerManager},
};

pub(crate) trait SimulationActor {
    fn next_step(&mut self) -> Step;
    fn peek_next_step(&self) -> Option<Jiffies>;
    fn submit(&mut self, event: Event);
}

pub(crate) struct Actors {
    pub(crate) network: Network,
    pub(crate) timers: TimerManager,
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

    pub(super) fn submit(&mut self, events: &mut [Event]) {
        for event in events.iter().cloned() {
            match event {
                e @ Event::TimerEvent { .. } => self.timers.submit(e),
                e @ Event::NetworkEvent { .. } => self.network.submit(e),
            }
        }
    }
}
