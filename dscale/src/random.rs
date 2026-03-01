//! Random number generation and probability distributions for DScale simulations.
//!
//! This module provides probability distributions used to model realistic
//! network characteristics such as variable latency, packet loss, and other
//! stochastic behaviors in distributed systems. All randomness is deterministic
//! and reproducible based on the simulation seed.

use rand::{Rng, SeedableRng, distr::Uniform, seq::IndexedRandom};
use rand_distr::{Bernoulli, Normal};

use crate::Jiffies;

pub type Seed = u64;

/// Probability distributions for modeling stochastic network behavior.
///
/// `Distributions` provides various probability distributions that can be used
/// to model realistic network characteristics in DScale simulations. Each
/// distribution generates values in [`Jiffies`] (simulation time units) and
/// is commonly used for latency modeling, though they can be applied to any
/// stochastic simulation parameter.
///
/// # Deterministic Randomness
///
/// All distributions use the simulation's deterministic random number generator,
/// ensuring that identical configurations with the same seed produce identical
/// results. This is crucial for reproducible simulation experiments.
///
/// # Common Applications
///
/// - **Network Latency**: Model variable message delivery times
/// - **Packet Loss**: Simulate unreliable network conditions
/// - **Jitter**: Add realistic variation to timing
/// - **Failure Models**: Probabilistic component failures
/// - **Load Variation**: Variable processing times
///
/// # Usage in Network Configuration
///
/// Distributions are primarily used with [`LatencyDescription`] to configure
/// network topology characteristics:
///
/// ```rust
/// use dscale::{SimulationBuilder, LatencyDescription, Distributions, Jiffies};
///
/// let simulation = SimulationBuilder::default()
///     .add_pool::<MyProcess>("servers", 3)
///     .latency_topology(&[
///         // Low-latency local network with small variation
///         LatencyDescription::WithinPool("servers",
///             Distributions::Uniform(Jiffies(1), Jiffies(5))),
///
///         // Internet latency with realistic variation
///         LatencyDescription::BetweenPools("clients", "servers",
///             Distributions::Normal(Jiffies(100), Jiffies(20))),
///
///         // Unreliable network with 10% packet loss
///         LatencyDescription::WithinPool("mobile",
///             Distributions::Bernoulli(0.9, Jiffies(50))),
///     ])
///     .build();
/// # struct MyProcess;
/// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
/// # impl dscale::ProcessHandle for MyProcess {
/// #     fn start(&mut self) {}
/// #     fn on_message(&mut self, from: dscale::Rank, message: dscale::MessagePtr) {}
/// #     fn on_timer(&mut self, id: dscale::TimerId) {}
/// # }
/// ```
///
/// [`Jiffies`]: crate::Jiffies
/// [`LatencyDescription`]: crate::LatencyDescription
#[derive(Copy, Clone)]
pub enum Distributions {
    Uniform(Jiffies, Jiffies),
    Bernoulli(f64, Jiffies),
    Normal(Jiffies, Jiffies),
}

pub struct Randomizer {
    rnd: rand::rngs::StdRng,
}

impl Randomizer {
    pub fn new(seed: Seed) -> Self {
        Self {
            rnd: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    pub fn random_usize(&mut self, d: Distributions) -> usize {
        match d {
            Distributions::Uniform(Jiffies(from), Jiffies(to)) => {
                let distr = Uniform::new_inclusive(from, to).expect("Invalid bounds");
                self.rnd.sample(distr)
            }
            Distributions::Bernoulli(p, Jiffies(val)) => {
                let distr = Bernoulli::new(p).expect("Invalid probability");
                if self.rnd.sample(distr) { val } else { 0 }
            }
            Distributions::Normal(Jiffies(mean), Jiffies(std_dev)) => {
                let distr = Normal::new(mean as f64, std_dev as f64).expect("Invalid parameters");
                self.rnd.sample(distr).max(0.0).round() as usize
            }
        }
    }

    pub fn choose_from_slice<'a, T: Copy>(&mut self, from: &[T]) -> T {
        from.choose(&mut self.rnd)
            .copied()
            .expect("Chose from empty slice")
    }
}
