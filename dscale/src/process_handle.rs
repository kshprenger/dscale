use crate::{MessagePtr, actors::timer_actor::TimerId};

/// Unique identifier for a process within the simulation. Ranks are assigned
/// sequentially starting from 0 in the order pools are added.
pub type Rank = usize;

/// Core trait for defining process behavior in the simulation.
///
/// Each process reacts to three kinds of events: startup, incoming messages,
/// and timer firings. Inside any handler you may call the global interaction
/// functions ([`crate::send_to`], [`crate::broadcast`], [`crate::schedule_timer_after`], etc.).
pub trait ProcessHandle {
    /// Called once when the simulation starts, before any messages are delivered.
    fn on_start(&mut self);

    /// Called when a message arrives from another process.
    fn on_message(&mut self, from: Rank, message: MessagePtr);

    /// Called when a previously scheduled timer fires.
    fn on_timer(&mut self, id: TimerId);
}

impl<T: ProcessHandle + ?Sized> ProcessHandle for Box<T> {
    fn on_start(&mut self) {
        (**self).on_start()
    }

    fn on_message(&mut self, from: Rank, message: MessagePtr) {
        (**self).on_message(from, message)
    }

    fn on_timer(&mut self, id: TimerId) {
        (**self).on_timer(id)
    }
}
