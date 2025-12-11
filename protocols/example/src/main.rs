use std::time::Instant;

use simulator::{Jiffies, Message, ProcessHandle, ProcessId, SimulationBuilder};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum ExampleMessage {
    Ping,
    Pong,
}

impl Message for ExampleMessage {
    fn VirtualSize(&self) -> usize {
        100
    }
}

struct ExampleProcess {
    self_id: ProcessId,
}

impl ExampleProcess {
    fn New() -> Self {
        Self { self_id: 0 }
    }
}

impl ProcessHandle<ExampleMessage> for ExampleProcess {
    fn Bootstrap(
        &mut self,
        assigned_id: ProcessId,
        outgoing: &mut simulator::OutgoingMessages<ExampleMessage>,
    ) {
        self.self_id = assigned_id;
        if assigned_id == 1 {
            outgoing.SendTo(2, ExampleMessage::Ping);
        }
    }

    fn OnMessage(
        &mut self,
        from: ProcessId,
        message: ExampleMessage,
        outgoing: &mut simulator::OutgoingMessages<ExampleMessage>,
    ) {
        if from == 1 && self.self_id == 2 {
            assert!(message == ExampleMessage::Ping);
            outgoing.SendTo(1, ExampleMessage::Pong);
            return;
        }

        if from == 2 && self.self_id == 1 {
            assert!(message == ExampleMessage::Pong);
            outgoing.SendTo(2, ExampleMessage::Ping);
            return;
        }
    }
}

fn main() {
    let start = Instant::now();

    let m = SimulationBuilder::NewFromFactory(|| ExampleProcess::New())
        .NetworkBandwidth(simulator::BandwidthType::Unbounded)
        .MaxLatency(Jiffies(10))
        .MaxTime(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    );

    let start = Instant::now();

    let m = SimulationBuilder::NewFromFactory(|| ExampleProcess::New())
        .NetworkBandwidth(simulator::BandwidthType::Bounded(5))
        .MaxLatency(Jiffies(10))
        .MaxTime(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    );
}
