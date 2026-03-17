mod bandwidth;
mod latency;

use std::sync::Arc;

pub use bandwidth::BandwidthDescription;
pub(crate) use bandwidth::BandwidthQueue;
pub(crate) use latency::LatencyQueue;
use log::debug;

use crate::GLOBAL_POOL;
use crate::MessagePtr;
use crate::Rank;
use crate::actor::EventSubmitter;
use crate::actor::SimulationActor;
use crate::destination::Destination;
use crate::event::Event;
use crate::now;
use crate::random::Randomizer;
use crate::random::Seed;
use crate::step::Step;
use crate::step::TimedStep;
use crate::time::Jiffies;
use crate::topology::Topology;

pub(crate) struct Network {
    seed: Seed,
    bandwidth_queue: BandwidthQueue,
    topology: Arc<Topology>,
}

impl Network {
    fn submit_single_message(
        &mut self,
        message: MessagePtr,
        source: Rank,
        destination: Destination,
    ) {
        let targets = match destination {
            Destination::BroadcastWithinPool(pool_name) => self.topology.list_pool(pool_name),
            Destination::To(to) => &[to],
        };

        debug!("Submitting message from {source}, targets of the message: {targets:?}",);

        targets.into_iter().copied().for_each(|target| {
            let timed_step = TimedStep {
                invocation_time: now() + Jiffies(1), // Without any latency message will arrive on next timepoint;
                step: Step::NetworkStep {
                    from: source,
                    to: target,
                    message: message.clone(),
                },
            };
            self.bandwidth_queue.push(timed_step);
        });
    }
}

impl Network {
    pub(crate) fn new(
        seed: Seed,
        bandwidth_type: BandwidthDescription,
        topology: Arc<Topology>,
    ) -> Self {
        Self {
            seed,
            bandwidth_queue: BandwidthQueue::new(
                bandwidth_type,
                topology.list_pool(GLOBAL_POOL).len(),
                LatencyQueue::new(Randomizer::new(seed), topology.clone()),
            ),
            topology,
        }
    }
}

impl SimulationActor for Network {
    fn next_step(&mut self) -> Step {
        self.bandwidth_queue
            .pop()
            .expect("Should not be empty")
            .step
    }

    fn peek_closest_step(&self) -> Option<Jiffies> {
        self.bandwidth_queue.peek_closest()
    }
}

impl EventSubmitter for Network {
    fn submit(&mut self, events: &mut Vec<Event>) {
        events.drain(..).for_each(|event| match event {
            Event::NetworkEvent { from, to, message } => {
                self.submit_single_message(message, from, to);
            }
            _ => unreachable!(),
        });
    }
}
