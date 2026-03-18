
use crate::{MessagePtr, time::timer_manager::TimerId};

pub type Rank = usize;

pub trait ProcessHandle: Sync + Send {
    fn start(&self);

    fn on_message(&self, from: Rank, message: MessagePtr);

    fn on_timer(&self, id: TimerId);
}
