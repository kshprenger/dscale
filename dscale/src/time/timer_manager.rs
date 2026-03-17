//! Timer management for process scheduling in DScale simulations.
//!
//! This module provides timer functionality that allows processes to schedule
//! delayed execution of callbacks. Timers are managed centrally by the simulation
//! engine and fire deterministically based on simulation time progression.

use std::cmp::Reverse;

use crate::{
    actor::{EventSubmitter, SimulationActor},
    event::Event,
    global, now,
    step::{Step, StepQueue, TimedStep},
    time::Jiffies,
};

/// Unique identifier for scheduled timers.
///
/// `TimerId` is a unique identifier returned when scheduling a timer using
/// [`schedule_timer_after`]. This identifier is passed back to the process
/// when the timer fires, allowing the process to distinguish between
/// different active timers.
///
/// Timer IDs are guaranteed to be unique within a single simulation run
/// and are generated using the global unique ID system.
///
/// # Usage
///
/// Timer IDs are primarily used in two contexts:
/// 1. **Scheduling**: Returned by [`schedule_timer_after`] to identify the timer
/// 2. **Handling**: Passed to [`ProcessHandle::on_timer`] when the timer fires
///
/// # Implementation Details
///
/// - Timer IDs are implemented as `usize` values
/// - IDs are generated using [`global_unique_id`] to ensure uniqueness
/// - Timer IDs are only valid within the simulation run that created them
/// - There is no built-in timer cancellation mechanism (implement cancellation logic in your process)
///
/// [`schedule_timer_after`]: crate::schedule_timer_after
/// [`ProcessHandle::on_timer`]: crate::ProcessHandle::on_timer
/// [`global_unique_id`]: crate::global_unique_id
pub type TimerId = usize;

pub(crate) fn next_timer_id() -> TimerId {
    global::global_unique_id()
}

#[derive(Default)]
pub(crate) struct TimerManager {
    working_timers: StepQueue,
}

impl SimulationActor for TimerManager {
    fn peek_closest_step(&self) -> Option<Jiffies> {
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
}

impl EventSubmitter for TimerManager {
    fn submit(&mut self, events: &mut Vec<Event>) {
        events.drain(..).for_each(|event| match event {
            Event::TimerEvent { to, id, fire_after } => {
                self.working_timers.push(Reverse(TimedStep {
                    invocation_time: now() + fire_after,
                    step: Step::TimerStep { to, id },
                }))
            }
            _ => unreachable!(),
        });
    }
}
