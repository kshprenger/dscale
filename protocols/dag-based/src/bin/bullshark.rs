use dag_based::bullshark::Bullshark;
use simulator::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};

fn main() {
    metrics::Clear();
    metrics::Set::<Vec<Jiffies>>("latency", Vec::new());
    metrics::Set::<usize>("timeouts-fired", 0);

    SimulationBuilder::NewFromFactory(|| Box::new(Bullshark::New()))
        .MaxLatency(Jiffies(5))
        .MaxTime(Jiffies(100000))
        .NICBandwidth(BandwidthType::Bounded(100))
        .ProcessInstances(60)
        .Seed(234565432345)
        .Build()
        .Run();

    println!(
        "Latency distribution: {:?}",
        metrics::Get::<Vec<Jiffies>>("latency").unwrap()
    );
    println!(
        "Timeouts fired: {}",
        metrics::Get::<usize>("timeouts-fired").unwrap()
    );
}
