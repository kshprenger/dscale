use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    ProcessHandle, Rank,
    actors::{
        Actors,
        network_actor::{BandwidthDescription, NetworkActor},
        timer_actor::TimerActor,
    },
    global,
    jiffy::Jiffies,
    random::Seed,
    runners::{SimulationRunner, scalable::ScalableRunner, simple::SimpleRunner, workers::Workers},
    simulation_flavor::SimulationFlavor,
    topology::{GLOBAL_POOL, LatencyDescription, LatencyTopology, PoolListing, Topology},
};

fn init_logger() {
    let _ = env_logger::Builder::from_default_env().try_init();
}

/// Builder for configuring and constructing a simulation.
///
/// Use the builder methods to add process pools, set network topology,
/// configure bandwidth, choose an execution mode (single-threaded or parallel),
/// and finally call [`SimulationBuilder::build`] to obtain a runnable simulation.
pub struct SimulationBuilder {
    seed: Seed,
    time_budget: Jiffies,
    proc_id: usize,
    handles: Vec<Option<Box<dyn ProcessHandle + Send>>>,
    pools: HashMap<String, Vec<Rank>>,
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
            handles: Vec::new(),
            pools: HashMap::default(),
            latency_topology: LatencyTopology::default(),
            bandwidth: BandwidthDescription::default(),
            flavor: SimulationFlavor::default(),
            safe_parallel_window: Jiffies(usize::MAX),
        }
    }
}

impl SimulationBuilder {
    /// Creates a named pool of `size` processes of type `P`.
    /// Every process is also added to [`GLOBAL_POOL`].
    pub fn add_pool<P: ProcessHandle + Default + Send + 'static>(
        mut self,
        name: &str,
        size: usize,
    ) -> SimulationBuilder {
        (0..size).for_each(|_| {
            let id = self.proc_id;
            self.proc_id += 1;
            self.handles.push(Some(Box::new(P::default())));
            self.pools.entry(name.to_string()).or_default().push(id);
            self.pools
                .entry(GLOBAL_POOL.to_string())
                .or_default()
                .push(id);
        });

        self
    }

    /// Sets the random seed for deterministic execution.
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the maximum simulation duration. The simulation stops when this time is reached.
    pub fn time_budget(mut self, time_budget: Jiffies) -> Self {
        self.time_budget = time_budget;
        self
    }

    /// Configures network latency between and within pools.
    /// Latency is symmetric: specifying A→B also sets B→A.
    pub fn latency_topology(mut self, descriptions: &[LatencyDescription]) -> Self {
        descriptions.iter().for_each(|d| {
            let (from, to, distr) = match d {
                LatencyDescription::WithinPool(name, distr) => (*name, *name, distr),
                LatencyDescription::BetweenPools(pool_from, pool_to, distr) => {
                    (*pool_from, *pool_to, distr)
                }
            };

            let from_vec: Vec<Rank> = self.pools.get(from).expect("No pool found").clone();

            let to_vec: Vec<Rank> = self.pools.get(to).expect("No pool found").clone();

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

    /// Configures per-process NIC bandwidth limits.
    pub fn nic_bandwidth(mut self, bandwidth: BandwidthDescription) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    /// Selects single-threaded execution mode (default).
    pub fn simple(mut self) -> Self {
        self.flavor = SimulationFlavor::Simple;
        self
    }

    /// Selects parallel execution mode using the given number of worker threads.
    pub fn parallel(mut self, cores: usize) -> Self {
        self.flavor = SimulationFlavor::Parallel(cores);
        self
    }

    /// Finalizes configuration and builds the simulation runner.
    pub fn build(mut self) -> Box<dyn SimulationRunner> {
        init_logger();

        let mut pool_listing = PoolListing::default();

        // Ensure latency_topology matrix is sized for all processes
        let n = self.proc_id;
        self.latency_topology.resize_with(n, || vec![None; n]);
        for row in &mut self.latency_topology {
            row.resize(n, None);
        }

        for (name, ids) in self.pools {
            pool_listing.insert(name, ids);
        }

        let topology = Topology::new_arc(pool_listing.clone(), self.latency_topology);
        let network_actor = NetworkActor::new(self.seed, self.bandwidth, topology.clone());
        let timers_actor = TimerActor::default();
        let actors = Actors {
            network: network_actor,
            timers: timers_actor,
        };

        global::configuration::setup_global_configuration(n);
        global::setup_shared_access(topology);

        match self.flavor {
            SimulationFlavor::Simple => {
                let procs: Vec<Box<dyn ProcessHandle>> = self
                    .handles
                    .into_iter()
                    .map(|opt| opt.expect("Uninitialized process slot") as Box<dyn ProcessHandle>)
                    .collect();
                Box::new(SimpleRunner::new(
                    actors,
                    self.time_budget,
                    procs,
                    self.seed,
                ))
            }
            SimulationFlavor::Parallel(cores) => {
                let procs: Vec<Arc<Mutex<dyn ProcessHandle + Send>>> = self
                    .handles
                    .into_iter()
                    .map(|opt| {
                        let handle = opt.expect("Uninitialized process slot");
                        Arc::new(Mutex::new(handle)) as Arc<Mutex<dyn ProcessHandle + Send>>
                    })
                    .collect();
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
