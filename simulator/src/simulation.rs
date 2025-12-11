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
    pub(crate) fn New(
        seed: random::Seed,
        max_steps: Jiffies,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: Vec<(ProcessId, P)>,
    ) -> Self {
        let _ = env_logger::try_init();

        Self {
            bandwidth_queue: BandwidthQueue::New(
                bandwidth_type,
                procs.len(),
                LatencyQueue::New(Randomizer::New(seed), max_network_latency),
            ),
            procs: procs.into_iter().collect(),
            metrics: Metrics::default(),
            global_time: Jiffies(0),
            max_steps: max_steps,
        }
    }

    pub fn Run(&mut self) -> Metrics {
        self.InitialStep();

        while self.KeepRunning() {
            if !self.Step() {
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
    fn SubmitMessages(&mut self, source: ProcessId, messages: Vec<(Destination, M)>) {
        messages.into_iter().for_each(|(destination, event)| {
            self.SubmitSingleMessage(event, source, destination, self.global_time + Jiffies(1));
        });
    }

    fn SubmitSingleMessage(
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

        debug!("Submited message, targets of the message: {targets:?}");
        targets.into_iter().for_each(|target| {
            self.bandwidth_queue
                .Push((base_arrival_time, (source, target, message.clone())));
        });
    }

    fn HandleOf(&mut self, process_id: ProcessId) -> &mut P {
        self.procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
    }

    fn KeepRunning(&mut self) -> bool {
        self.global_time < self.max_steps
    }

    fn InitialStep(&mut self) {
        for id in self.procs.keys().copied().collect::<Vec<ProcessId>>() {
            debug!("Executing initial step for {id}");
            let mut outgoing_messages = OutgoingMessages::New();
            self.HandleOf(id).Bootstrap(id, &mut outgoing_messages);
            self.SubmitMessages(id, outgoing_messages.0);
        }
    }

    fn Step(&mut self) -> bool {
        let next_event = self.bandwidth_queue.pop();

        match next_event {
            BandwidthQueueOptions::None => false,
            BandwidthQueueOptions::MessageArrivedByLatency => true, // Do nothing
            BandwidthQueueOptions::Some(message) => {
                self.FastForwardClock(message.0);
                self.ExecuteProcessStep(message.1);
                true
            }
        }
    }

    fn FastForwardClock(&mut self, time: Jiffies) {
        debug_assert!(self.global_time <= time, "Time is not monotonous");
        self.global_time = time;
        debug!("Global time now: {time}");
    }

    fn ExecuteProcessStep(&mut self, step: ProcessStep<M>) {
        self.metrics.TrackEvent();

        let source = step.0;
        let dest = step.1;
        let message = step.2;

        let mut outgoing_messages = OutgoingMessages::New();

        debug!("Executing step for {} | Message Source: {}", dest, source);
        self.HandleOf(dest)
            .OnMessage(source, message, &mut outgoing_messages);
        self.SubmitMessages(dest, outgoing_messages.0);
    }
}
