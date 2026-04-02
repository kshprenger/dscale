mod bandwidth;
mod latency;

use std::sync::Arc;

pub use bandwidth::BandwidthConfig;
pub(crate) use bandwidth::BandwidthQueue;
pub(crate) use latency::LatencyQueue;
use log::debug;

use crate::GLOBAL_POOL;
use crate::MessagePtr;
use crate::Rank;
use crate::actors::SimulationActor;
use crate::destination::Destination;
use crate::event::Event;
use crate::jiffy::Jiffies;
use crate::now;
use crate::random::Randomizer;
use crate::random::Seed;
use crate::step::Step;
use crate::step::TimedStep;
use crate::topology::Topology;

pub(crate) struct NetworkActor {
    bandwidth_queue: BandwidthQueue,
    topology: Arc<Topology>,
}

impl NetworkActor {
    fn submit_single_message(
        &mut self,
        message: MessagePtr,
        source: Rank,
        destination: Destination,
    ) {
        let targets = match destination {
            Destination::BroadcastWithinPool(pool_name) => self.topology.list_pool(pool_name),
            Destination::Target(rank) => &[rank],
        };

        debug!("Submitting steps P{source} -> P{targets:?}");
        let base_time = now() + Jiffies(1);
        for &target in targets {
            let timed_step = TimedStep {
                invocation_time: base_time,
                step: Step::NetworkStep {
                    source: source,
                    target: target,
                    message: message.clone(),
                },
            };
            self.bandwidth_queue.push(timed_step);
        }
    }
}

impl NetworkActor {
    pub(crate) fn new(
        seed: Seed,
        bandwidth_type: BandwidthConfig,
        topology: Arc<Topology>,
    ) -> Self {
        Self {
            bandwidth_queue: BandwidthQueue::new(
                bandwidth_type,
                topology.list_pool(GLOBAL_POOL).len(),
                LatencyQueue::new(Randomizer::new(seed), topology.clone()),
            ),
            topology,
        }
    }
}

impl SimulationActor for NetworkActor {
    fn next_step(&mut self) -> Step {
        self.bandwidth_queue
            .pop()
            .expect("Should not be empty")
            .step
    }

    fn peek_next_step(&self) -> Option<Jiffies> {
        self.bandwidth_queue.peek_closest()
    }

    fn submit(&mut self, event: Event) {
        match event {
            Event::NetworkEvent {
                source,
                destination,
                message,
            } => {
                self.submit_single_message(message, source, destination);
            }
            _ => unreachable!(),
        }
    }
}
