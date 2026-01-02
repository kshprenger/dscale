#![allow(non_snake_case)]

use std::time::Instant;

use matrix::*;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum PingPongMessage {
    Ping,
    Pong,
}

impl Message for PingPongMessage {
    fn VirtualSize(&self) -> usize {
        match self {
            PingPongMessage::Ping => 50,
            PingPongMessage::Pong => 100,
        }
    }
}

struct ExampleProcess {
    timer_id: TimerId,
}

impl ExampleProcess {
    fn New() -> Self {
        Self { timer_id: 0 }
    }
}

impl ProcessHandle for ExampleProcess {
    fn Bootstrap(&mut self, _configuration: Configuration) {
        if CurrentId() == 1 {
            self.timer_id = ScheduleTimerAfter(Jiffies(100));
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        assert!(message.Is::<PingPongMessage>());
        let m = message.As::<PingPongMessage>();

        if from == 1 && CurrentId() == 2 {
            assert!(*m == PingPongMessage::Ping);
            Debug!("Sending Pong");
            SendTo(1, PingPongMessage::Pong);
            return;
        }

        if from == 2 && CurrentId() == 1 {
            assert!(*m == PingPongMessage::Pong);
            Debug!("Sending Ping");
            SendTo(2, PingPongMessage::Ping);
            return;
        }
    }

    fn OnTimer(&mut self, id: TimerId) {
        assert!(id == self.timer_id);
        SendTo(2, PingPongMessage::Ping);
    }
}

fn main() {
    let start = Instant::now();

    SimulationBuilder::NewFromFactory(|| Box::new(ExampleProcess::New()))
        .NICBandwidth(matrix::BandwidthType::Unbounded)
        .MaxLatency(Jiffies(10))
        .TimeBudget(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!("Done, elapsed: {:?}", start.elapsed());

    let start = Instant::now();

    SimulationBuilder::NewFromFactory(|| Box::new(ExampleProcess::New()))
        .NICBandwidth(matrix::BandwidthType::Bounded(5))
        .MaxLatency(Jiffies(10))
        .TimeBudget(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!("Done, elapsed: {:?}", start.elapsed());
}
