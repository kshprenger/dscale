use dag_based::sparse_bullshark::SparseBullshark;
use matrix::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};

fn main() {
    anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
    anykv::Set::<usize>("D", 7);

    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<SparseBullshark>("Validators", 100)
        .LatencyTopology(&[LatencyDescription::WithinPool(
            "Validators",
            Distributions::Uniform(Jiffies(0), Jiffies(400)),
        )])
        .TimeBudget(Jiffies(600000))
        .NICBandwidth(BandwidthDescription::Unbounded)
        .Seed(234565432345)
        .Build();

    sim.Run();

    println!(
        "Vertices ordered: {}",
        anykv::Get::<(f64, usize)>("avg_latency").1
    );
}
