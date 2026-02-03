use crate::{MessagePtr, TimerId};

pub(crate) enum MatrixMessage {
    NetworkMessage(MessagePtr),
    Timer(TimerId),
}
