#![allow(non_snake_case)]

use dscale::{global::anykv, *};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct DataMessage {
    pub real_payload: u64,
}

impl Message for DataMessage {
    fn VirtualSize(&self) -> usize {
        1000
    }
}

#[derive(Default)]
pub struct Sender {}

impl ProcessHandle for Sender {
    fn Start(&mut self) {
        // Start sending immediately
        ScheduleTimerAfter(Jiffies(1));
    }

    fn OnMessage(&mut self, _from: ProcessId, _message: MessagePtr) {
        // Sender doesn't receive messages
    }

    fn OnTimer(&mut self, _id: TimerId) {
        SendTo(2, DataMessage { real_payload: 42 });
        anykv::Modify::<usize>("messages_sent", |x| *x += 1);
        ScheduleTimerAfter(Jiffies(1));
    }
}

#[derive(Default)]
pub struct Receiver {}

impl ProcessHandle for Receiver {
    fn Start(&mut self) {}

    fn OnMessage(&mut self, _from: ProcessId, message: MessagePtr) {
        let _ = message.As::<DataMessage>();
        anykv::Modify::<usize>("messages_received", |x| *x += 1);
    }

    fn OnTimer(&mut self, _id: TimerId) {}
}
