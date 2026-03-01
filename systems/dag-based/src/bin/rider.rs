use dag_based::rider::DAGRider;
use dscale::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::kv,
    time::Jiffies,
};

fn main() {
    let mut sim = SimulationBuilder::default()
        .add_pool::<DAGRider>("Validators", 53)
        .latency_topology(&[LatencyDescription::WithinPool(
            "Validators",
            Distributions::Normal(Jiffies(50), Jiffies(10)),
        )])
        .time_budget(Jiffies(3600_000))
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .seed(123)
        .build();

    kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

    sim.run();

    let ordered = kv::get::<(f64, usize)>("avg_latency").1;
    let avg_latency = kv::get::<(f64, usize)>("avg_latency").0;
    println!("ordered: {ordered}, avg_latency: {avg_latency}")
}
