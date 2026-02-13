#![allow(non_snake_case)]

use dscale::{global::anykv, *};

// This demo shows 2 data centers: in first one there are pingers processes,
// in the second one - pongers processes. Pingers send ping to a single random pong process and vice versa.

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Ping;
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Pong;

impl Message for Ping {}
impl Message for Pong {}

#[derive(Default)]
pub struct PingProcess {}

impl ProcessHandle for PingProcess {
    fn Start(&mut self) {
        SendRandomFromPool("Pongers", Ping);
        anykv::Modify::<usize>("pings", |p| *p += 1);
    }

    fn OnMessage(&mut self, _from: ProcessId, message: MessagePtr) {
        let _ = message.Is::<Pong>();
        SendRandomFromPool("Pongers", Ping);
        anykv::Modify::<usize>("pings", |p| *p += 1);
    }

    fn OnTimer(&mut self, _id: TimerId) {}
}

#[derive(Default)]
pub struct PongProcess {}

impl ProcessHandle for PongProcess {
    fn Start(&mut self) {}

    fn OnMessage(&mut self, _from: ProcessId, message: MessagePtr) {
        let _ = message.Is::<Ping>();
        SendRandomFromPool("Pingers", Pong);
        anykv::Modify::<usize>("pongs", |p| *p += 1);
    }

    fn OnTimer(&mut self, _id: TimerId) {}
}
