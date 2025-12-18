use crate::{Destination, Jiffies, Message, ProcessId, access::Access};

pub struct CoreAccess<M: Message> {
    pub(crate) scheduled_events: Vec<(Destination, M)>,
    pub(crate) current_time: Jiffies,
}

impl<M: Message> CoreAccess<M> {
    pub(crate) fn New(current_time: Jiffies) -> Self {
        Self {
            scheduled_events: Vec::new(),
            current_time,
        }
    }
}

impl<M: Message> Access<M> for CoreAccess<M> {
    fn Broadcast(&mut self, message: M) {
        self.scheduled_events
            .push((Destination::Broadcast, message));
    }

    fn SendTo(&mut self, to: ProcessId, message: M) {
        self.scheduled_events.push((Destination::To(to), message));
    }

    fn SendSelf(&mut self, message: M) {
        self.scheduled_events.push((Destination::SendSelf, message));
    }

    fn CurrentTime(&self) -> Jiffies {
        self.current_time
    }
}
