use std::collections::HashMap;

use crate::{
    communication::{Event, EventType},
    history::{ExecutionHistory, ProcessStep},
    process::ProcessId,
};

#[derive(Clone, Default)]
pub(crate) struct Metrics {
    pub events_total: usize,
    pub timeout_distribution: HashMap<ProcessId, usize>,
    pub execution_history: ExecutionHistory,
}

impl Metrics {
    pub(crate) fn track_step(&mut self, step: ProcessStep) {
        self.track_event(step.0, &step.1);
        self.execution_history.push(step);
    }
}

impl Metrics {
    fn track_timeout(&mut self, id: ProcessId) {
        if let Some(count) = self.timeout_distribution.get_mut(&id) {
            *count += 1;
        } else {
            self.timeout_distribution.insert(id, 1);
        }
    }

    fn track_event(&mut self, id: ProcessId, event: &Event) {
        self.events_total += 1;
        match event.event_type {
            EventType::Timeout => {
                self.track_timeout(id);
            }
            EventType::Message(_) => {}
        }
    }
}
