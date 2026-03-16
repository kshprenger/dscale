mod bandwidth;
mod latency;

use std::sync::{Arc, Mutex};

pub use bandwidth::BandwidthDescription;
pub(crate) use bandwidth::BandwidthQueue;
pub(crate) use latency::LatencyQueue;
use log::debug;

use crate::Message;
use crate::MessagePtr;
use crate::Rank;
use crate::actor::EventSubmitter;
use crate::actor::SimulationActor;
use crate::destination::Destination;
use crate::event::DScaleMessage;
use crate::global::configuration;
use crate::message::Event as RoutedMessage;
use crate::message::Step as ProcessStep;
use crate::now;
use crate::nursery::Nursery;
use crate::random::Randomizer;
use crate::random::Seed;
use crate::time::Jiffies;
use crate::topology::Topology;

pub(crate) type NetworkActor = Arc<Mutex<Network>>;

pub(crate) struct Network {
    seed: Seed,
    bandwidth_queue: BandwidthQueue,
    topology: Arc<Topology>,
    nursery: Arc<Nursery>,
}

impl Network {
    fn submit_single_message(
        &mut self,
        message: Arc<dyn Message>,
        source: Rank,
        destination: Destination,
    ) {
        let targets = match destination {
            Destination::BroadcastWithinPool(pool_name) => self.topology.list_pool(pool_name),
            Destination::To(to) => &[to],
        };

        debug!("Submitting message from {source}, targets of the message: {targets:?}",);

        targets.into_iter().copied().for_each(|target| {
            let routed_message = RoutedMessage {
                arrival_time: now() + Jiffies(1), // Without any latency message will arrive on next timepoint;
                step: ProcessStep {
                    source,
                    dest: target,
                    message: message.clone(),
                },
            };
            self.bandwidth_queue.push(routed_message);
        });
    }

    fn execute_process_step(&mut self, step: ProcessStep) {
        let source = step.source;
        let dest = step.dest;
        let message = step.message;

        self.nursery.deliver(
            source,
            dest,
            DScaleMessage::NetworkMessage(MessagePtr(message)),
        );
    }
}

impl Network {
    pub(crate) fn new(
        seed: Seed,
        bandwidth_type: BandwidthDescription,
        topology: Arc<Topology>,
        nursery: Arc<Nursery>,
    ) -> Self {
        Self {
            seed,
            bandwidth_queue: BandwidthQueue::new(
                bandwidth_type,
                nursery.size(),
                LatencyQueue::new(Randomizer::new(seed), topology.clone()),
            ),
            topology,
            nursery,
        }
    }
}

impl SimulationActor for Network {
    fn start(&mut self) {
        self.nursery.keys().for_each(|id| {
            configuration::setup_local_configuration(id, self.seed);
            self.nursery.start_single(id);
        });
    }

    fn step(&mut self) {
        let next_event = self.bandwidth_queue.pop();

        match next_event {
            None => {}
            Some(message) => {
                self.execute_process_step(message.step);
            }
        }
    }

    fn peek_closest(&self) -> Option<Jiffies> {
        self.bandwidth_queue.peek_closest()
    }
}

impl EventSubmitter for Network {
    type Event = (Rank, Destination, Arc<dyn Message>);

    fn submit(&mut self, events: &mut Vec<Self::Event>) {
        events.drain(..).for_each(|(from, destination, message)| {
            self.submit_single_message(message, from, destination);
        });
    }
}
