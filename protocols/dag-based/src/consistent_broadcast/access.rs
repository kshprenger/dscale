use simulator::{Access, Message, ProcessId};

use crate::consistent_broadcast::message::BCBMessage;

pub(super) struct BCBAccess<'a, M, A>
where
    M: Message,
    A: Access<BCBMessage<M>>,
{
    pub(super) scheduled_broadcasts: Vec<M>,
    outer_access: &'a mut A,
}

impl<'a, M, A> BCBAccess<'a, M, A>
where
    M: Message,
    A: Access<BCBMessage<M>>,
{
    pub(super) fn Wrap(access: &'a mut A) -> Self {
        Self {
            scheduled_broadcasts: Vec::new(),
            outer_access: access,
        }
    }
}

impl<'a, M, A> Access<M> for BCBAccess<'a, M, A>
where
    M: Message,
    A: Access<BCBMessage<M>>,
{
    fn CurrentTime(&self) -> simulator::Jiffies {
        self.outer_access.CurrentTime()
    }

    fn SendTo(&mut self, to: ProcessId, message: M) {
        self.outer_access.SendTo(to, BCBMessage::Skip(message));
    }

    fn SendSelf(&mut self, message: M) {
        self.outer_access.SendSelf(BCBMessage::Skip(message));
    }

    fn Broadcast(&mut self, message: M) {
        self.scheduled_broadcasts.push(message);
    }
}
