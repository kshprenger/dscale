use simulator::*;

use crate::dag_utils::RoundBasedDAG;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum BullsharkMessage {}

impl Message for BullsharkMessage {
    fn VirtualSize(&self) -> usize {
        todo!()
    }
}

struct Vertex {}

struct Bullshark {
    dag: RoundBasedDAG,
}

impl Bullshark {
    fn New() -> Self {
        Self {
            dag: RoundBasedDAG::New(69),
        }
    }
}

impl ProcessHandle<BullsharkMessage> for Bullshark {
    fn Bootstrap(
        &mut self,
        assigned_id: ProcessId,
        outgoing: &mut simulator::OutgoingMessages<BullsharkMessage>,
    ) {
        todo!()
    }

    fn OnMessage(
        &mut self,
        from: ProcessId,
        message: BullsharkMessage,
        outgoing: &mut simulator::OutgoingMessages<BullsharkMessage>,
    ) {
        todo!();
    }
}

impl Bullshark {
    fn TryAdvanceRound(&mut self) {
        todo!()
    }
}

fn main() {
    todo!()
}
