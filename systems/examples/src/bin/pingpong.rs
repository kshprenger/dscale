use std::time::Instant;

use examples::pingpong::PingPongProcess;
use matrix::{global::anykv, *};

fn main() {
    let start = Instant::now();
    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<PingPongProcess>("ExamplePool", 2)
        .NICBandwidth(BandwidthDescription::Unbounded)
        .LatencyTopology(&[LatencyDescription::WithinPool(
            "ExamplePool",
            Distributions::Uniform(Jiffies(0), Jiffies(10)),
        )])
        .TimeBudget(Jiffies(100_000_000))
        .Seed(5)
        .Build();

    anykv::Set::<usize>("pings", 0);
    anykv::Set::<usize>("pongs", 0);

    sim.Run();

    println!(
        "Done, elapsed: {:?}. Pings sent: {}, Pongs sent: {}",
        start.elapsed(),
        anykv::Get::<usize>("pings"),
        anykv::Get::<usize>("pongs"),
    );
}
