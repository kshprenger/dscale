use std::cmp::Reverse;

use crate::{
    actor::SimulationActor,
    event::Event,
    global, now,
    step::{Step, StepQueue, TimedStep},
    time::Jiffies,
};

pub type TimerId = usize;

pub(crate) fn next_timer_id() -> TimerId {
    global::global_unique_id()
}

#[derive(Default)]
pub(crate) struct TimerManager {
    working_timers: StepQueue,
}

impl SimulationActor for TimerManager {
    fn peek_next_step(&self) -> Option<Jiffies> {
        self.working_timers
            .peek()
            .map(|entry| entry.0.invocation_time)
    }

    fn next_step(&mut self) -> Step {
        self.working_timers
            .pop()
            .expect("Should not be empty")
            .0
            .step
    }

    fn submit(&mut self, event: Event) {
        match event {
            Event::TimerEvent { to, id, fire_after } => {
                self.working_timers.push(Reverse(TimedStep {
                    invocation_time: now() + fire_after,
                    step: Step::TimerStep { to, id },
                }))
            }
            _ => unreachable!(),
        }
    }
}
