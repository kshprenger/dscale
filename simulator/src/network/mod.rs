mod bandwidth;
mod latency;

use std::cell::RefMut;
use std::collections::BTreeMap;
use std::rc::Rc;

pub(crate) use bandwidth::BandwidthQueue;
pub(crate) use bandwidth::BandwidthQueueOptions;
pub use bandwidth::BandwidthType;
pub(crate) use latency::LatencyQueue;
use log::debug;

use crate::Configuration;
use crate::Destination;
use crate::Message;
use crate::MessagePtr;
use crate::ProcessHandle;
use crate::ProcessId;
use crate::access;
use crate::actor::SimulationActor;
use crate::communication::ProcessStep;
use crate::communication::RoutedMessage;
use crate::process::ProcessPool;
use crate::random::Randomizer;
use crate::random::Seed;
use crate::time::Jiffies;
use crate::time::Now;

pub(crate) struct Network {
    seed: Seed,
    bandwidth_queue: BandwidthQueue,
    procs: Rc<ProcessPool>,
}

impl Network {
    fn SubmitSingleMessage(
        &mut self,
        message: Rc<dyn Message>,
        source: ProcessId,
        destination: Destination,
        base_arrival_time: Jiffies,
    ) {
        let targets = match destination {
            Destination::Broadcast => self.procs.Keys().copied().collect::<Vec<ProcessId>>(),
            Destination::To(to) => vec![to],
        };

        debug!("Submitting message from {source}, targets of the message: {targets:?}",);

        targets.into_iter().for_each(|target| {
            let routed_message = RoutedMessage {
                arrival_time: base_arrival_time,
                step: ProcessStep {
                    source,
                    dest: target,
                    message: message.clone(),
                },
            };
            self.bandwidth_queue.Push(routed_message);
        });
    }

    fn ExecuteProcessStep(&mut self, step: ProcessStep) {
        let source = step.source;
        let dest = step.dest;
        let message = step.message;

        debug!(
            "Executing step for process {} | Message Source: {}",
            dest, source
        );

        access::SetProcess(dest);

        self.procs
            .Get(dest)
            .OnMessage(source, MessagePtr::New(message));
    }
}

impl Network {
    pub(crate) fn New(
        seed: Seed,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: Rc<ProcessPool>,
    ) -> Self {
        Self {
            seed,
            bandwidth_queue: BandwidthQueue::New(
                bandwidth_type,
                procs.Size(),
                LatencyQueue::New(Randomizer::New(seed), max_network_latency),
            ),
            procs,
        }
    }

    pub(crate) fn SubmitMessages(
        &mut self,
        messages: &mut Vec<(ProcessId, Destination, Rc<dyn Message>)>,
    ) {
        messages
            .drain(..)
            .into_iter()
            .for_each(|(from, destination, message)| {
                self.SubmitSingleMessage(message, from, destination, Now() + Jiffies(1));
            });
    }
}

impl SimulationActor for Network {
    fn Start(&mut self) {
        self.procs.IterMut().for_each(|(id, mut handle)| {
            debug!("Executing initial step for {id}");

            let config = Configuration {
                seed: self.seed,
                proc_num: self.procs.Keys().len(),
            };

            access::SetProcess(*id);

            handle.Bootstrap(config);
        });
    }

    fn Step(&mut self) {
        let next_event = self.bandwidth_queue.Pop();

        match next_event {
            BandwidthQueueOptions::None => {}
            BandwidthQueueOptions::MessageArrivedByLatency => {}
            BandwidthQueueOptions::Some(message) => {
                self.ExecuteProcessStep(message.step);
            }
        }
    }

    fn PeekClosest(&self) -> Option<Jiffies> {
        self.bandwidth_queue.PeekClosest()
    }
}
