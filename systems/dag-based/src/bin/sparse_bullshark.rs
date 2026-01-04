use dag_based::sparse_bullshark::SparseBullshark;
use matrix::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};

fn main() {
    metrics::Set::<Vec<Jiffies>>("latency", Vec::new());
    metrics::Set::<usize>("timeouts-fired", 0);

    SimulationBuilder::NewDefault()
        .AddPool("Validators", 100, || SparseBullshark::New(10))
        .MaxLatency(Jiffies(400))
        .TimeBudget(Jiffies(600000))
        .NICBandwidth(BandwidthType::Unbounded)
        .Seed(234565432345)
        .Build()
        .Run();

    println!(
        "Vertices ordered: {}",
        metrics::Get::<Vec<Jiffies>>("latency").len()
    );

    println!(
        "Timeouts fired: {}",
        metrics::Get::<usize>("timeouts-fired")
    );
}
