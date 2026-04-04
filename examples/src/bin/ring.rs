use std::time::Instant;

use dscale::{global::kv, *};
use examples::ring::Ring;

fn main() {
    let ring_size = 1000;

    let mut sim = SimulationBuilder::default()
        .add_pool::<Ring>("RingPool", ring_size)
        .vnic_bandwidth(BandwidthConfig::Unbounded)
        .within_pool_latency("RingPool", Distributions::Uniform(Jiffies(1), Jiffies(10)))
        .time_budget(Jiffies(100_000_000))
        .simple()
        .seed(5)
        .build();

    kv::set::<usize>("passes", 0);

    let start = Instant::now();
    sim.run_full_budget();
    let elapsed = start.elapsed();

    println!(
        "Elapsed: {:?}. Passes: {}",
        elapsed,
        kv::get::<usize>("passes"),
    );

    println!(
        "DScale performance: steps/sec {:.2}",
        kv::get::<usize>("passes") as f64 / elapsed.as_secs_f64()
    );
}
