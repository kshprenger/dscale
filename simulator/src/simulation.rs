use std::collections::HashMap;

use crate::{
    communication::{Destination, Event, EventDeliveryQueue, EventId, EventType},
    fault::NetworkController,
    metrics::{self, Metrics},
    process::{ProcessHandle, ProcessId},
    random::{self, Randomizer},
    time::Jiffies,
};

pub struct Simulation {
    network_controller: NetworkController,
    procs: HashMap<ProcessId, (Box<dyn ProcessHandle>, EventDeliveryQueue)>,
    metrics: Metrics,
    current_process: Option<ProcessId>,
    global_time: Jiffies,
    max_steps: Jiffies,
    next_event: EventId,
}

impl Simulation {
    pub(crate) fn new(
        seed: random::Seed,
        max_steps: Jiffies,
        max_network_latency: Jiffies,
    ) -> Self {
        Self {
            network_controller: NetworkController::new(Randomizer::new(seed), max_network_latency),
            procs: HashMap::new(),
            metrics: Metrics::default(),
            current_process: None,
            global_time: 0,
            max_steps: max_steps,
            next_event: 0,
        }
    }

    pub(crate) fn submit_event_after(
        &mut self,
        event_type: EventType,
        destination: Destination,
        after: Jiffies,
    ) -> EventId {
        let next_event_id = self.get_next_event_id();
        let will_arrive_at = self.calculate_arrival_time(after);

        let event = Event {
            id: next_event_id,
            event_type,
        };

        let targets = match destination {
            Destination::Broadcast => self.procs.keys().copied().collect::<Vec<ProcessId>>(),
            Destination::SendSelf => vec![self.curr_process()],
        };

        targets.into_iter().for_each(|target| {
            self.devilery_queue_of(target)
                .push(event.clone(), will_arrive_at);
        });

        next_event_id
    }

    pub(crate) fn cancel_event(&mut self, event: &Event) {
        self.devilery_queue_of(self.curr_process()).remove(event);
    }

    pub(crate) fn add_processes(&mut self, procs: Vec<Box<dyn ProcessHandle>>) {
        procs.into_iter().enumerate().for_each(|(id, proc)| {
            self.procs.insert(id, (proc, EventDeliveryQueue::new()));
        });
    }

    pub(crate) fn run(&mut self) -> metrics::Metrics {
        self.initial_step();

        while self.keep_running() {
            if !self.step() {
                panic!("Deadlock")
            }
        }

        self.metrics.clone()
    }
}

impl Simulation {
    fn curr_process(&self) -> ProcessId {
        self.current_process.expect("No current process")
    }

    fn devilery_queue_of(&mut self, process_id: ProcessId) -> &mut EventDeliveryQueue {
        &mut self
            .procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
            .1
    }

    fn handle_of(&mut self, process_id: ProcessId) -> &mut Box<dyn ProcessHandle> {
        &mut self
            .procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
            .0
    }

    fn keep_running(&mut self) -> bool {
        self.tick();
        self.global_time < self.max_steps
    }

    fn tick(&mut self) {
        self.global_time += 1;
    }

    fn calculate_arrival_time(&mut self, after: Jiffies) -> Jiffies {
        after + self.global_time + self.network_controller.introduce_random_latency()
    }

    fn get_next_event_id(&mut self) -> EventId {
        self.next_event += 1;
        self.next_event
    }

    fn initial_step(&mut self) {
        self.procs.iter_mut().for_each(|(id, (process_handle, _))| {
            self.current_process = Some(*id);
            process_handle.init();
        });
    }

    fn step(&mut self) -> bool {
        let next_events = self.choose_next_events();
        if next_events.is_empty() {
            return false;
        }
        self.deliver_events(next_events);
        return true;
    }

    fn deliver_events(&mut self, events: Vec<(ProcessId, Event)>) {
        events.into_iter().for_each(|(target, event)| {
            self.current_process = Some(target);
            let produced_messages = self.handle_of(target).on_event(event);
            produced_messages
                .into_iter()
                .for_each(|(destination, message)| {
                    self.submit_event_after(EventType::Message(message), destination, 0);
                });
        })
    }

    fn choose_next_events(&mut self) -> Vec<(ProcessId, Event)> {
        self.procs
            .iter_mut()
            .filter(|(_, (_, candidate_queue))| !candidate_queue.is_empty())
            .map(|(candidate, (_, candidate_queue))| {
                (candidate, candidate_queue.pop().expect("Queue is empty"))
            })
            .filter(|(_, (_, arrival_time))| *arrival_time == self.global_time)
            .map(|(candidate, (event, _))| (candidate.clone(), event))
            .collect()
    }
}
