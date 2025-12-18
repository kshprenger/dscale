use dag_based::{bullshark::Bullshark, consistent_broadcast::ByzantineConsistentBroadcast};
use simulator::{BandwidthType, Jiffies, SimulationBuilder};
fn main() {
    for procs in (4..1000).step_by(10) {
        let mut sim = SimulationBuilder::NewFromFactory(|| {
            ByzantineConsistentBroadcast::Wrap(Bullshark::New())
        })
        .MaxLatency(Jiffies(133))
        .MaxTime(Jiffies(23440))
        .NetworkBandwidth(BandwidthType::Unbounded)
        .ProcessInstances(procs)
        .Seed(procs as u64)
        .Build();
        let metrics = sim.Run();
        println!("{}, {}", procs, metrics.events_total)
    }
}
