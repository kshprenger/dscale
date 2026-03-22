use dscale::{global::kv, *};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct BroadcastMessage {
    pub data: u64,
}

impl Message for BroadcastMessage {}

#[derive(Default)]
pub struct BroadcastProcess {}

impl ProcessHandle for BroadcastProcess {
    fn on_start(&self) {
        // Process with rank 0 starts the broadcast
        if rank() == 0 {
            schedule_timer_after(Jiffies(100));
        }
    }

    fn on_message(&self, from: Rank, message: MessagePtr) {
        let msg = message.as_type::<BroadcastMessage>();
        debug_process!("Received broadcast from {}: data={}", from, msg.data);

        assert_eq!(msg.data, 42);

        kv::modify::<usize>("broadcast_received", |x| *x += 1);
    }

    fn on_timer(&self, _id: TimerId) {
        debug_process!("Broadcasting value 42");
        broadcast(BroadcastMessage { data: 42 });
        schedule_timer_after(Jiffies(100));
    }
}
