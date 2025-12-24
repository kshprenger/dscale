use dag_based::bullshark::Bullshark;
use simulator::{BandwidthType, Jiffies, SimulationBuilder};
fn main() {
    let mut sim = SimulationBuilder::NewFromFactory(|| Bullshark::New())
        .MaxLatency(Jiffies(500))
        .MaxTime(Jiffies(10000_000))
        .NetworkBandwidth(BandwidthType::Bounded(100))
        .ProcessInstances(60)
        .Seed(234565432345)
        .Build();

    let _ = sim.Run();
}
