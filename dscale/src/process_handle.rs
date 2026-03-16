use crate::{MessagePtr, actors::timer_actor::TimerId};

pub type Rank = usize;

pub trait ProcessHandle {
    fn on_start(&mut self);

    fn on_message(&mut self, from: Rank, message: MessagePtr);

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
