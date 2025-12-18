use crate::{Access, Message, ProcessId, process::Configuration};

pub trait ProcessHandle<M: Message> {
    /// This methods provides initial configuration to the process.
    /// It is also requires process to schedule some initial messages.
    fn Bootstrap(&mut self, configuration: Configuration, access: &mut impl Access<M>);

    /// Deliver message with source process
    fn OnMessage(&mut self, from: ProcessId, message: M, access: &mut impl Access<M>);
}
