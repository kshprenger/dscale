use dag_based::sparse_bullshark::SparseBullshark;
use matrix::{BandwidthType, SimulationBuilder, global::anykv, time::Jiffies};

fn main() {
    anykv::Set::<Vec<Jiffies>>("latency", Vec::new());
    anykv::Set::<usize>("timeouts-fired", 0);
    anykv::Set::<usize>("D", 10);

    SimulationBuilder::NewDefault()
        .AddPool::<SparseBullshark>("Validators", 100)
        .MaxLatency(Jiffies(400))
        .TimeBudget(Jiffies(600000))
        .NICBandwidth(BandwidthType::Unbounded)
        .Seed(234565432345)
        .Build()
        .Run();

    println!(
        "Vertices ordered: {}",
        anykv::Get::<Vec<Jiffies>>("latency").len()
    );

    println!("Timeouts fired: {}", anykv::Get::<usize>("timeouts-fired"));
}
