use std::time::Instant;

use dscale::{global::kv, *};
use examples::timers::LazyPingPong;

fn main() {
    let mut sim = SimulationBuilder::default()
        .add_pool::<LazyPingPong>("TimerDemoPool", 2)
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .latency_topology(&[LatencyDescription::WithinPool(
            "TimerDemoPool",
            Distributions::Uniform(Jiffies(10), Jiffies(50)),
        )])
        .time_budget(Jiffies(10_000))
        .seed(42)
        .build();

    kv::set::<usize>("heartbeats", 0);
    kv::set::<usize>("pings_received", 0);
    kv::set::<usize>("pongs_received", 0);

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    let heartbeats = kv::get::<usize>("heartbeats");
    let pings = kv::get::<usize>("pings_received");
    let pongs = kv::get::<usize>("pongs_received");

    println!();
    println!("Simulation completed in: {:?}", elapsed);
    println!("Heartbeats: {}", heartbeats);
    println!("Pings received: {}", pings);
    println!("Pongs received: {}", pongs);

    assert_eq!(pings, 5);
    assert_eq!(pongs, 5);
    assert_eq!(heartbeats, 19);
}
