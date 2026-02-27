//! Network topology configuration for latency simulation.
//!
//! This module provides types and structures for configuring network latency
//! characteristics between processes in DScale simulations. It supports
//! modeling different latency patterns within process pools and between
//! different pools to create realistic network topologies.

use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use crate::{ProcessId, random::Distributions};

pub(crate) type LatencyTopology = BTreeMap<(ProcessId, ProcessId), Distributions>;
pub(crate) type PoolListing = HashMap<String, Vec<ProcessId>>;

/// Default pool for all processes within simulation.
/// Broadcasts by default use this pool.
pub const GLOBAL_POOL: &str = "global_pool";

/// Describes network latency characteristics for different process relationships.
///
/// `LatencyDescription` allows you to configure different latency patterns
/// based on the relationship between communicating processes. This enables
/// realistic modeling of network topologies where processes in the same
/// datacenter have lower latency than those separated by geographic distance.
///
/// # Topology Modeling
///
/// The latency system supports two primary relationship types:
/// - **Within Pool**: Latency between processes in the same named pool
/// - **Between Pools**: Latency between processes in different named pools
///
/// Each relationship can be configured with different probability distributions
/// to model various network characteristics like jitter, packet loss, and
/// varying network conditions.
///
/// # Usage in Simulation Configuration
///
/// Latency descriptions are provided to [`SimulationBuilder::latency_topology`]
/// as an array, allowing you to specify multiple latency relationships:
///
/// ```rust
/// use dscale::{SimulationBuilder, LatencyDescription, Distributions, Jiffies};
///
/// let simulation = SimulationBuilder::default()
///     .add_pool::<MyProcess>("datacenter_a", 3)
///     .add_pool::<MyProcess>("datacenter_b", 2)
///     .add_pool::<MyProcess>("mobile_clients", 5)
///     .latency_topology(&[
///         // Low latency within datacenters
///         LatencyDescription::WithinPool("datacenter_a",
///             Distributions::Uniform(Jiffies(1), Jiffies(3))),
///         LatencyDescription::WithinPool("datacenter_b",
///             Distributions::Uniform(Jiffies(1), Jiffies(3))),
///
///         // Higher latency between datacenters
///         LatencyDescription::BetweenPools("datacenter_a", "datacenter_b",
///             Distributions::Normal(Jiffies(50), Jiffies(10))),
///
///         // Variable latency to mobile clients
///         LatencyDescription::BetweenPools("datacenter_a", "mobile_clients",
///             Distributions::Normal(Jiffies(100), Jiffies(30))),
///         LatencyDescription::BetweenPools("datacenter_b", "mobile_clients",
///             Distributions::Normal(Jiffies(120), Jiffies(35))),
///
///         // High variability within mobile clients (shared medium)
///         LatencyDescription::WithinPool("mobile_clients",
///             Distributions::Bernoulli(0.9, Jiffies(80))), // 10% packet loss
///     ])
///     .build();
/// # struct MyProcess;
/// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
/// # impl dscale::ProcessHandle for MyProcess {
/// #     fn start(&mut self) {}
/// #     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
/// #     fn on_timer(&mut self, id: dscale::TimerId) {}
/// # }
/// ```
///
/// [`SimulationBuilder::latency_topology`]: crate::SimulationBuilder::latency_topology
pub enum LatencyDescription {
    /// Configures latency for messages within a single process pool.
    ///
    /// This variant specifies the latency characteristics for communication
    /// between any two processes that belong to the same named pool. It's
    /// typically used to model local network conditions, such as processes
    /// within the same datacenter, rack, or local network segment.
    ///
    /// # Parameters
    ///
    /// * `&'static str` - The name of the pool (must match a pool created with [`add_pool`])
    /// * [`Distributions`] - The probability distribution for latency values
    ///
    /// # Common Use Cases
    ///
    /// - **Datacenter Networks**: Low, consistent latency between co-located servers
    /// - **Local Clusters**: Minimal latency within tightly coupled systems
    /// - **Rack-Local**: Very low latency between processes on same hardware rack
    /// - **Shared Medium**: Variable latency on shared network segments
    ///
    WithinPool(&'static str, Distributions),

    /// Configures latency for messages between two different process pools.
    ///
    /// This variant specifies the latency characteristics for communication
    /// between processes in two different named pools. It models the network
    /// characteristics between different locations, network segments, or
    /// administrative domains.
    ///
    /// # Parameters
    ///
    /// * `&'static str` - The name of the first pool
    /// * `&'static str` - The name of the second pool
    /// * [`Distributions`] - The probability distribution for latency values
    ///
    /// # Bidirectional Application
    ///
    /// The latency configuration is automatically applied in both directions.
    /// Specifying `BetweenPools("A", "B", distribution)` also configures
    /// the latency from pool "B" to pool "A" with the same distribution.
    ///
    /// # Common Use Cases
    ///
    /// - **Geographic Distribution**: Higher latency between distant datacenters
    /// - **Network Tiers**: Different latency between client and server tiers
    /// - **Cloud Regions**: Inter-region communication delays
    /// - **Cross-Network**: Latency across different network providers
    ///
    BetweenPools(&'static str, &'static str, Distributions),
}

pub(crate) struct Topology {
    pool_listing: PoolListing,
    latency_topology: LatencyTopology,
}

impl Topology {
    pub(crate) fn new_shared(
        pool_listing: PoolListing,
        latency_topology: LatencyTopology,
    ) -> Rc<Self> {
        Rc::new(Self {
            pool_listing,
            latency_topology,
        })
    }

    pub(crate) fn get_distribution(&self, from: ProcessId, to: ProcessId) -> Distributions {
        self.latency_topology
            .get(&(from, to))
            .copied()
            .expect("No distr found")
    }

    pub(crate) fn list_pool(&self, pool_name: &str) -> &[usize] {
        self.pool_listing.get(pool_name).expect("Invalid pool name")
    }
}
