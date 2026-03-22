use std::cmp::Reverse;

use crate::{
    actors::SimulationActor,
    event::Event,
    jiffy::Jiffies,
    now,
    step::{Step, StepQueue, TimedStep},
};

pub type TimerId = usize;

#[derive(Default)]
pub(crate) struct TimerActor {
    working_timers: StepQueue,
}

impl SimulationActor for TimerActor {
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
            Event::TimerEvent {
                rank,
                id,
                fire_after,
            } => self.working_timers.push(Reverse(TimedStep {
                invocation_time: now() + fire_after,
                step: Step::TimerStep { rank, id },
            })),
            _ => unreachable!(),
        }
    }
}
