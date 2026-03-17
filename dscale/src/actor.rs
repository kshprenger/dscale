use crate::{event::Event, step::Step, time::Jiffies};

pub(crate) trait SimulationActor {
    fn next_step(&mut self) -> Step;
    fn peek_closest_step(&self) -> Option<Jiffies>;
}

pub(crate) trait EventSubmitter {
    fn submit(&mut self, events: &mut Vec<Event>);
}
