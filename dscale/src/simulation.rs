
use std::{
    process::exit,
    sync::{Arc, Mutex},
    usize,
};

use log::{error, info};

use crate::{
    actor::SharedActor,
    global,
    network::{BandwidthDescription, Network},
    nursery::{HandlerMap, Nursery},
    progress::Bar,
    random,
    time::{Jiffies, timer_manager::TimerManager},
    topology::{LatencyTopology, PoolListing, Topology},
};

pub struct Simulation {
    actors: Vec<SharedActor>,
    time_budget: Jiffies,
    progress_bar: Bar,
}

impl Simulation {
    pub(crate) fn new(
        seed: random::Seed,
        time_budget: Jiffies,
        bandwidth: BandwidthDescription,
        latency_topology: LatencyTopology,
        pool_listing: PoolListing,
        procs: HandlerMap,
    ) -> Self {
        let topology = Topology::new_shared(pool_listing.clone(), latency_topology);
        let nursery = Nursery::new(procs);

        let network_actor = Arc::new(Mutex::new(Network::new(
            seed,
            bandwidth,
            topology.clone(),
            nursery.clone(),
        )));

        let timers_actor = Arc::new(Mutex::new(TimerManager::new(nursery.clone())));

        global::init_globals();
        global::configuration::setup_global_configuration(nursery.size());

        let actors: Vec<SharedActor> = vec![network_actor, timers_actor];

        Self {
            actors,
            time_budget,
            progress_bar: Bar::new(time_budget),
        }
    }

    pub fn run(&mut self) {
        self.start();

        while global::now() < self.time_budget {
            self.step();
        }

        // For small simulations progress bar is not fullfilling
        self.progress_bar.finish();

        info!("Looks good! ヽ('ー`)ノ");
    }
}

impl Simulation {
    fn start(&mut self) {
        self.actors.iter_mut().for_each(|actor| {
            actor.lock().expect("Actor lock poisoned").start();
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

impl Drop for Simulation {
    fn drop(&mut self) {
        global::drop_all(); // Clear globals
    }
}
