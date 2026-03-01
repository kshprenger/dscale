use dscale::{
    global::{kv, configuration::process_number},
    *,
};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct RingMessage {}

impl Message for RingMessage {}

#[derive(Default)]
pub struct Ring {}

fn pass_next() {
    kv::modify::<usize>("passes", |p| *p += 1);
    send_to(((rank() + 1) % process_number()) + 1, RingMessage {});
}

impl ProcessHandle for Ring {
    fn start(&mut self) {
        if rank() == 1 {
            pass_next();
        }
    }

    fn on_message(&mut self, _from: Rank, _message: MessagePtr) {
        pass_next();
    }

    fn on_timer(&mut self, _id: TimerId) {}
}
