use simulator::{Message, ProcessId};

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct BCBMessageId {
    pub(super) process_id: ProcessId,
    pub(super) message_id: usize,
}

#[derive(Clone)]
pub enum BCBMessage<M: Message> {
    // Broadcast
    Initiate((BCBMessageId, M)),
    Signature(BCBMessageId),
    Certificate(BCBMessageId),
    // Other
    Skip(M),
}

impl<M: Message> Message for BCBMessage<M> {
    fn VirtualSize(&self) -> usize {
        0
    }
}
