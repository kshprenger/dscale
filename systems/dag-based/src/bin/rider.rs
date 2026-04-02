use dag_based::rider::DAGRider;
use dscale::{
    BandwidthConfig, Distributions, Jiffies, SimulationBuilder, Threads, global::kv,
};

fn main() {
    let mut sim = SimulationBuilder::default()
        .add_pool::<DAGRider>("Validators", 1000)
        .within_pool_latency(
            "Validators",
            Distributions::Normal {
                mean: Jiffies(50),
                std_dev: Jiffies(5),
                low: Jiffies(40),
                high: Jiffies(80),
            },
        )
        .time_budget(Jiffies(3600))
        .vnic_bandwidth(BandwidthConfig::Unbounded)
        .seed(123)
        .parallel(Threads::All)
        .build();

    kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

    sim.run_full_budget();

    let ordered = kv::get::<(f64, usize)>("avg_latency").1;
    let avg_latency = kv::get::<(f64, usize)>("avg_latency").0;
    println!("ordered: {ordered}, avg_latency: {avg_latency}")
}
