use async_trait::async_trait;

use crate::communication::{Event, Message};
use std::collections::HashSet;

pub type ProcessId = usize;

tokio::task_local! {
     pub static PROCESS_ID: ProcessId;
}

#[async_trait]
pub trait Process {
    async fn on_event(&mut self, m: Event) -> HashSet<Message>;
}
