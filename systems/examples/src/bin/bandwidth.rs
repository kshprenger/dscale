use std::time::Instant;

use dscale::{global::anykv, *};
use examples::bandwidth::{Receiver, Sender};

fn main() {
    println!("=== Bandwidth Example ===\n");

    let unbounded_count = run_unbounded();
    println!("Unbounded: messages received = {}\n", unbounded_count);

    let bounded_count = run_bounded();
    println!("Bounded: messages received = {}\n", bounded_count);

    // Assert that bounded receives fewer messages due to bandwidth constraints
    assert!(
        bounded_count < unbounded_count,
        "Bounded ({}) should receive fewer messages than unbounded ({})",
        bounded_count,
        unbounded_count
    );
}

fn run_unbounded() -> usize {
    anykv::Set::<usize>("messages_sent", 0);
    anykv::Set::<usize>("messages_received", 0);

    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<Sender>("Senders", 1)
        .AddPool::<Receiver>("Receivers", 1)
        .NICBandwidth(BandwidthDescription::Unbounded)
        .LatencyTopology(&[LatencyDescription::BetweenPools(
            "Senders",
            "Receivers",
            Distributions::Uniform(Jiffies(10), Jiffies(10)),
        )])
        .TimeBudget(Jiffies(10_000))
        .Seed(42)
        .Build();

    let start = Instant::now();
    sim.Run();
    let elapsed = start.elapsed();

    let sent = anykv::Get::<usize>("messages_sent");
    let received = anykv::Get::<usize>("messages_received");
    println!(
        "  Elapsed: {:?}, sent: {}, received: {}",
        elapsed, sent, received
    );

    received
}

fn run_bounded() -> usize {
    anykv::Set::<usize>("messages_sent", 0);
    anykv::Set::<usize>("messages_received", 0);

    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<Sender>("Senders", 1)
        .AddPool::<Receiver>("Receivers", 1)
        // Very low bandwidth: 1 byte per jiffy (messages will queue up)
        .NICBandwidth(BandwidthDescription::Bounded(1))
        .LatencyTopology(&[
            LatencyDescription::WithinPool(
                "Senders",
                Distributions::Uniform(Jiffies(1), Jiffies(1)),
            ),
            LatencyDescription::WithinPool(
                "Receivers",
                Distributions::Uniform(Jiffies(1), Jiffies(1)),
            ),
            LatencyDescription::BetweenPools(
                "Senders",
                "Receivers",
                Distributions::Uniform(Jiffies(10), Jiffies(10)),
            ),
        ])
        .TimeBudget(Jiffies(10_000))
        .Seed(42)
        .Build();

    let start = Instant::now();
    sim.Run();
    let elapsed = start.elapsed();

    let sent = anykv::Get::<usize>("messages_sent");
    let received = anykv::Get::<usize>("messages_received");
    println!(
        "  Elapsed: {:?}, sent: {}, received: {}",
        elapsed, sent, received
    );

    received
}
