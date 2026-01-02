use std::cell::RefCell;

use crate::{MessagePtr, ProcessId, process::Configuration, time::timer::TimerId};

pub(crate) type UniqueProcessHandle = Box<dyn ProcessHandle>;
pub(crate) type MutableProcessHandle = RefCell<UniqueProcessHandle>;

pub trait ProcessHandle {
    // This methods provides initial configuration to the process.
    // It is also requires process to schedule some initial messages.
    fn Bootstrap(&mut self, configuration: Configuration);

    // Deliver message with source process
    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr);

    // Fire timer with id that was returned on ScheduleTimerAfter() call
    fn OnTimer(&mut self, #[allow(unused)] id: TimerId) {
        // Default - No timers
    }
}
