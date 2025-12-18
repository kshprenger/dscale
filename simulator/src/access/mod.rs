mod core_access;

pub use core_access::CoreAccess;

use crate::{Jiffies, ProcessId, communication::Message};

// Single network layer access
pub trait Access<M: Message> {
    fn Broadcast(&mut self, message: M);
    fn SendTo(&mut self, to: ProcessId, message: M);
    fn SendSelf(&mut self, message: M);
    fn CurrentTime(&self) -> Jiffies;
}
