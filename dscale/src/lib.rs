//! Deterministic & fast simulation framework for distributed systems.
//!
//! DScale provides a single-threaded or parallel, event-driven simulation engine
//! that models network latency, bandwidth constraints, and process execution.
//! Simulations are fully deterministic when seeded, making them reproducible
//! for testing and benchmarking.
//!
//! The main workflow:
//! 1. Implement [`ProcessHandle`] for your process logic.
//! 2. Define messages implementing [`Message`].
//! 3. Configure the simulation with [`SimulationBuilder`].
//! 4. Call `run_X()` on the result of [`SimulationBuilder::build`].

mod actors;
mod alloc;
mod destination;
mod event;
/// Global simulation state: clock, configuration, key-value store, and process interaction functions.
pub mod global;
/// Helper utilities for simulation processes.
pub mod helpers;
mod jiffy;
mod message;
mod process_handle;
mod random;
mod runners;
mod simulation_builder;
mod simulation_flavor;
mod step;
mod topology;

pub use message::Message;
pub use message::MessagePtr;

pub use process_handle::ProcessHandle;
pub use process_handle::Rank;

pub use simulation_builder::SimulationBuilder;

pub use global::broadcast;
pub use global::broadcast_within_pool;
pub use global::choose_from_pool;
pub use global::global_unique_id;
pub use global::list_pool;
pub use global::now;
pub use global::rank;
pub use global::schedule_timer_after;
pub use global::send_random_from_pool;
pub use global::send_to;

pub use actors::network_actor::BandwidthConfig;

pub use topology::GLOBAL_POOL;

pub use random::Distributions;

pub use actors::timer_actor::TimerId;
pub use jiffy::Jiffies;
pub use runners::RunStatus;
pub use runners::SimulationRunner;
pub use runners::threads::Threads;
