use std::{collections::HashMap, sync::Arc};

use crate::{
    ProcessHandle, Rank,
    actor::Actors,
    global,
    network::{BandwidthDescription, Network},
    random::Seed,
    runners::{
        SimulationRunner, deterministic::DeterministicRunner, scalable::ScalableRunner,
        workers::Workers,
    },
    simulation_flavor::SimulationFlavor,
    time::{Jiffies, timer_manager::TimerManager},
    topology::{GLOBAL_POOL, LatencyDescription, LatencyTopology, PoolListing, Topology},
};

fn init_logger() {
    let _ = env_logger::Builder::from_default_env().try_init();
}

pub struct SimulationBuilder {
    seed: Seed,
    time_budget: Jiffies,
    proc_id: usize,
    pools: HashMap<String, Vec<(Rank, Arc<dyn ProcessHandle>)>>,
    latency_topology: LatencyTopology,
    bandwidth: BandwidthDescription,
    flavor: SimulationFlavor,
    safe_parallel_window: Jiffies,
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self {
            seed: Seed::default(),
            time_budget: Jiffies::default(),
            proc_id: 0,
            pools: HashMap::default(),
            latency_topology: LatencyTopology::default(),
            bandwidth: BandwidthDescription::default(),
            flavor: SimulationFlavor::default(),
            safe_parallel_window: Jiffies(usize::MAX),
        }
    }
}

impl SimulationBuilder {
    pub fn add_pool<P: ProcessHandle + Default + Send + 'static>(
        mut self,
        name: &str,
        size: usize,
    ) -> SimulationBuilder {
        (0..size).for_each(|_| {
            let id = self.proc_id;
            self.proc_id += 1;
            let handle = Arc::new(P::default());
            self.add_to_pool::<P>(name, id, handle.clone());
            self.add_to_pool::<P>(GLOBAL_POOL, id, handle.clone());
        });

        self
    }

    fn add_to_pool<P: ProcessHandle + Default + 'static>(
        &mut self,
        name: &str,
        id: usize,
        handle: Arc<dyn ProcessHandle>,
    ) {
        let pool = self.pools.entry(name.to_string()).or_default();
        pool.push((id, handle));
    }

    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    pub fn time_budget(mut self, time_budget: Jiffies) -> Self {
        self.time_budget = time_budget;
        self
    }

    pub fn latency_topology(mut self, descriptions: &[LatencyDescription]) -> Self {
        descriptions.iter().for_each(|d| {
            let (from, to, distr) = match d {
                LatencyDescription::WithinPool(name, distr) => (*name, *name, distr),
                LatencyDescription::BetweenPools(pool_from, pool_to, distr) => {
                    (*pool_from, *pool_to, distr)
                }
            };

            let from_vec: Vec<Rank> = self
                .pools
                .get(from)
                .expect("No pool found")
                .iter()
                .map(|(id, _)| *id)
                .collect();

            let to_vec: Vec<Rank> = self
                .pools
                .get(to)
                .expect("No pool found")
                .iter()
                .map(|(id, _)| *id)
                .collect();

            let cartesian_product = from_vec
                .iter()
                .flat_map(|x| to_vec.iter().map(move |y| (*x, *y)));

            let cartesian_product_backwards = from_vec
                .iter()
                .flat_map(|x| to_vec.iter().map(move |y| (*y, *x)));

            // Ensure matrix is large enough
            let max_rank = from_vec
                .iter()
                .chain(to_vec.iter())
                .copied()
                .max()
                .unwrap_or(0)
                + 1;
            if self.latency_topology.len() < max_rank {
                self.latency_topology
                    .resize_with(max_rank, || vec![None; max_rank]);
            }
            for row in &mut self.latency_topology {
                if row.len() < max_rank {
                    row.resize(max_rank, None);
                }
            }

            cartesian_product.for_each(|(from, to)| {
                self.latency_topology[from][to] = Some(distr.clone());
            });

            cartesian_product_backwards.for_each(|(from, to)| {
                self.latency_topology[from][to] = Some(distr.clone());
            });

            self.safe_parallel_window =
                std::cmp::min(self.safe_parallel_window, distr.safe_window())
        });
        self
    }

    pub fn nic_bandwidth(mut self, bandwidth: BandwidthDescription) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn deterministic(mut self) -> Self {
        self.flavor = SimulationFlavor::Deterministic;
        self
    }

    pub fn parallel(mut self, cores: usize) -> Self {
        self.flavor = SimulationFlavor::Parallel(cores);
        self
    }

    pub fn build(mut self) -> Box<dyn SimulationRunner> {
        init_logger();

        let mut pool_listing = PoolListing::default();
        let mut procs: Vec<Option<Arc<dyn ProcessHandle>>> = vec![None; self.proc_id];

        // Ensure latency_topology matrix is sized for all processes
        let n = self.proc_id;
        self.latency_topology.resize_with(n, || vec![None; n]);
        for row in &mut self.latency_topology {
            row.resize(n, None);
        }

        for (name, pool) in self.pools {
            let mut ids = Vec::new();
            for (id, handle) in pool {
                ids.push(id);
                procs[id] = Some(handle);
            }
            pool_listing.insert(name, ids);
        }

        let procs: Vec<Arc<dyn ProcessHandle>> = procs
            .into_iter()
            .map(|opt| opt.expect("Uninitialized process slot"))
            .collect();

        let topology = Topology::new_arc(pool_listing.clone(), self.latency_topology);
        let network_actor = Network::new(self.seed, self.bandwidth, topology.clone());
        let timers_actor = TimerManager::default();
        let actors = Actors {
            network: network_actor,
            timers: timers_actor,
        };

        global::configuration::setup_global_configuration(procs.len());
        global::setup_shared_access(topology);

        match self.flavor {
            SimulationFlavor::Deterministic => {
                let workers = Workers::new(procs, 1, self.seed);
                Box::new(DeterministicRunner::new(actors, self.time_budget, workers))
            }
            SimulationFlavor::Parallel(cores) => {
                let workers = Workers::new(procs, cores, self.seed);
                Box::new(ScalableRunner::new(
                    actors,
                    self.time_budget,
                    workers,
                    self.safe_parallel_window,
                ))
            }
        }
    }
}
