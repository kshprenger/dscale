use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{Rank, random::Distributions};

pub(crate) type LatencyTopology = Vec<Vec<Option<Distributions>>>;
pub(crate) type PoolListing = FxHashMap<String, Vec<Rank>>;

/// Name of the implicit pool that contains every process.
pub const GLOBAL_POOL: &str = "global_pool";

/// Describes latency rules for the network topology.
pub enum LatencyRule {
    /// Latency between processes within the same named pool.
    WithinPool(&'static str, Distributions),

    /// Latency between processes in two different pools (symmetric).
    BetweenPools(&'static str, &'static str, Distributions),
}

#[derive(Debug)]
pub(crate) struct Topology {
    pool_listing: PoolListing,
    latency_topology: LatencyTopology,
}

impl Topology {
    pub(crate) fn new_arc(
        pool_listing: PoolListing,
        latency_topology: LatencyTopology,
    ) -> Arc<Self> {
        Arc::new(Self {
            pool_listing,
            latency_topology,
        })
    }

    pub(crate) fn get_distribution(&self, from: Rank, to: Rank) -> Distributions {
        self.latency_topology[from][to].expect("No distr found")
    }

    pub(crate) fn list_pool(&self, pool_name: &str) -> &[usize] {
        self.pool_listing.get(pool_name).expect("Invalid pool name")
    }
}
