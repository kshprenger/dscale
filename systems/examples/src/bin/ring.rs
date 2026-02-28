use std::time::Instant;

use dscale::{global::anykv, *};
use examples::ring::Ring;

fn main() {
    let ring_size = 1000;

    let mut sim = SimulationBuilder::default()
        .add_pool::<Ring>("RingPool", ring_size)
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .latency_topology(&[LatencyDescription::WithinPool(
            "RingPool",
            Distributions::Uniform(Jiffies(1), Jiffies(10)),
        )])
        .time_budget(Jiffies(100_000_000))
        .seed(5)
        .build();

    anykv::set::<usize>("passes", 0);

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    println!(
        "Elapsed: {:?}. Passes: {}",
        elapsed,
        anykv::get::<usize>("passes"),
    );

    println!(
        "DScale performance: steps/sec {:.2}",
        anykv::get::<usize>("passes") as f64 / elapsed.as_secs_f64()
    );
}
