use std::{process::exit, sync::Arc, usize};

use log::{error, info};

use crate::{
    ProcessHandle,
    actor::SimulationActor,
    global,
    network::{BandwidthDescription, Network},
    progress::Bar,
    random,
    runner::SimulationRunner,
    time::{Jiffies, timer_manager::TimerManager},
    topology::{LatencyTopology, PoolListing, Topology},
};

pub struct DeterministicRunner {
    actors: Vec<Box<dyn SimulationActor>>,
    time_budget: Jiffies,
    procs: Vec<Arc<dyn ProcessHandle>>,
    progress_bar: Bar,
}

impl DeterministicRunner {
    pub(crate) fn new(
        seed: random::Seed,
        time_budget: Jiffies,
        bandwidth: BandwidthDescription,
        latency_topology: LatencyTopology,
        pool_listing: PoolListing,
        procs: Vec<Arc<dyn ProcessHandle>>,
    ) -> Self {
        let topology = Topology::new_arc(pool_listing.clone(), latency_topology);
        let network_actor = Box::new(Network::new(seed, bandwidth, topology.clone()));
        let timers_actor = Box::new(TimerManager::default());

        global::configuration::setup_global_configuration(procs.len() - 1);

        let actors: Vec<Box<dyn SimulationActor>> = vec![network_actor, timers_actor];

        Self {
            actors,
            time_budget,
            progress_bar: Bar::new(time_budget),
            procs,
        }
    }
}

impl SimulationRunner for DeterministicRunner {
    fn run_full_budget(&mut self) {
        self.start();

        while global::now() < self.time_budget {
            self.step();
        }

        self.progress_bar.finish();

        info!("Looks good! ヽ('ー`)ノ");
    }
}

impl DeterministicRunner {
    fn start(&mut self) {
        self.actors.iter_mut().for_each(|actor| {
            global::schedule();
        });
    }

    fn step(&mut self) {
        match self.peek_closest() {
            None => {
                error!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                exit(1)
            }
            Some((future, actor)) => {
                global::fast_forward_clock(future);
                actor.lock().expect("Actor lock poisoned").step();
                global::schedule();
                self.progress_bar
                    .make_progress(future.min(self.time_budget));
            }
        }
    }

    fn peek_closest(&mut self) -> Option<(Jiffies, SharedActor)> {
        let mut min_time = Jiffies(usize::MAX);
        let mut sha: Option<SharedActor> = None;
        for actor in self.actors.iter() {
            actor
                .lock()
                .expect("Actor lock poisoned")
                .peek_closest()
                .map(|time| {
                    if time < min_time {
                        min_time = time;
                        sha = Some(actor.clone())
                    }
                });
        }

        Some((min_time, sha?))
    }
}
