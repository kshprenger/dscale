use std::collections::HashMap;

use log::debug;

use crate::{
    OutgoingMessages,
    communication::{Destination, Message},
    metrics::Metrics,
    network_condition::{BandwidthQueue, BandwidthQueueOptions, BandwidthType, LatencyQueue},
    process::{ProcessHandle, ProcessId},
    random::{self, Randomizer},
    time::Jiffies,
};

pub struct Simulation<P, M>
where
    P: ProcessHandle<M>,
    M: Message,
{
    bandwidth_queue: BandwidthQueue<M>,
    procs: HashMap<ProcessId, P>,
    metrics: Metrics,
    global_time: Jiffies,
    max_steps: Jiffies,
}

pub(crate) type ProcessStep<M> = (ProcessId, ProcessId, M);

impl<P, M> Simulation<P, M>
where
    P: ProcessHandle<M>,
    M: Message,
{
    pub(crate) fn new(
        seed: random::Seed,
        max_steps: Jiffies,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: Vec<(ProcessId, P)>,
    ) -> Self {
        let _ = env_logger::try_init();

        Self {
            bandwidth_queue: BandwidthQueue::new(
                bandwidth_type,
                procs.len(),
                LatencyQueue::new(Randomizer::new(seed), max_network_latency),
            ),
            procs: procs.into_iter().collect(),
            metrics: Metrics::default(),
            global_time: Jiffies(0),
            max_steps: max_steps,
        }
    }

    pub fn run(&mut self) -> Metrics {
        self.initial_step();

        while self.keep_running() {
            if !self.step() {
                panic!("Deadlock")
            }
        }

        self.metrics.clone()
    }
}

impl<P, M> Simulation<P, M>
where
    P: ProcessHandle<M>,
    M: Message,
{
    fn submit_messages(&mut self, source: ProcessId, messages: Vec<(Destination, M)>) {
        messages.into_iter().for_each(|(destination, event)| {
            // Happy path -> schedule on the next step
            self.submit_message(event, source, destination, self.global_time + Jiffies(1));
        });
    }

    fn submit_message(
        &mut self,
        message: M,
        source: ProcessId,
        destination: Destination,
        base_arrival_time: Jiffies,
    ) {
        let targets = match destination {
            Destination::Broadcast => self.procs.keys().copied().collect::<Vec<ProcessId>>(),
            Destination::To(to) => vec![to],
            Destination::SendSelf => vec![source],
        };

        debug!("Submited message, targets: {targets:?}");

        targets.into_iter().for_each(|target| {
            self.bandwidth_queue
                .push((base_arrival_time, (source, target, message.clone())));
        });
    }

    fn handle_of(&mut self, process_id: ProcessId) -> &mut P {
        self.procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
    }

    fn keep_running(&mut self) -> bool {
        self.global_time < self.max_steps
    }

    fn initial_step(&mut self) {
        for id in self.procs.keys().copied().collect::<Vec<ProcessId>>() {
            let mut outgoing_messages = OutgoingMessages::new();
            self.handle_of(id).bootstrap(id, &mut outgoing_messages);
            self.submit_messages(id, outgoing_messages.0);
        }
    }

    fn step(&mut self) -> bool {
        let next_event = self.bandwidth_queue.pop();

        match next_event {
            BandwidthQueueOptions::None => false,
            BandwidthQueueOptions::MessageArrivedByLatency => true, // Do nothing
            BandwidthQueueOptions::Some(message) => {
                self.set_global_time(message.0);
                self.execute_process_step(message.1);
                true
            }
        }
    }

    fn set_global_time(&mut self, time: Jiffies) {
        debug_assert!(self.global_time <= time);
        debug!("Global time now: {}", time);
        self.global_time = time;
    }

    fn execute_process_step(&mut self, step: ProcessStep<M>) {
        debug!("Executing step. Source: {}, Dest: {}", step.0, step.1);
        self.metrics.track_event();

        let source = step.0;
        let dest = step.1;
        let message = step.2;

        let mut outgoing_messages = OutgoingMessages::new();
        self.handle_of(dest)
            .on_message(source, message, &mut outgoing_messages);
        self.submit_messages(dest, outgoing_messages.0);
    }
}
