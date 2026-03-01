//! Global configuration access for simulation parameters.
//!
//! This module provides access to simulation-wide and process-specific configuration
//! values. It manages both global settings that apply to all processes and local
//! settings that are specific to individual processes, such as random seeds.
//!
//! The configuration system uses the global key-value store internally and provides
//! type-safe access to commonly used configuration parameters.

use crate::{Rank, global::kv, random::Seed, rank};

pub(crate) fn setup_global_configuration(proc_num: usize) {
    kv::set::<usize>("proc_num", proc_num)
}

pub(crate) fn setup_local_configuration(id: Rank, base_seed: Seed) {
    // Prevent resonance between procs by changing seed a little bit
    kv::set::<u64>(&format!("seeds/{}", id), base_seed + id as u64)
}

/// Returns the random seed for the currently executing process.
///
/// Each process in the simulation receives a unique random seed derived from
/// the base simulation seed. This ensures that random number generation is
/// deterministic and reproducible while avoiding correlation between processes.
///
/// The seed is calculated by adding the process ID to the base simulation seed,
/// which prevents resonance effects between processes that might occur if all
/// processes used the same seed.
///
/// # Context
///
/// This function must be called from within a process context (i.e., during
/// the execution of [`ProcessHandle`] methods).
///
/// [`ProcessHandle`]: crate::ProcessHandle
///
/// # Returns
///
/// The unique random seed for the current process as a `u64`.
pub fn seed() -> Seed {
    kv::get::<u64>(&format!("seeds/{}", rank()))
}

/// Returns the total number of processes in the simulation.
///
/// This function provides access to the total count of all processes across
/// all pools in the current simulation. This value is set during simulation
/// setup and remains constant throughout the simulation run.
///
/// # Context
///
/// This function can be called from any context within the simulation,
/// including from within [`ProcessHandle`] methods.
///
/// [`ProcessHandle`]: crate::ProcessHandle
///
/// # Returns
///
/// The total number of processes in the simulation.
pub fn process_number() -> usize {
    kv::get::<usize>("proc_num")
}
