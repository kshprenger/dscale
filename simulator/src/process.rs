use crate::{Message, OutgoingMessages};

pub type ProcessId = usize;

pub trait ProcessHandle<M: Message> {
    /// Should schedule some initial events
    fn init(&mut self, outgoing: &mut OutgoingMessages<M>);

    /// Deliver event with source process
    fn on_message(&mut self, from: ProcessId, message: M, outgoing: &mut OutgoingMessages<M>);
}
