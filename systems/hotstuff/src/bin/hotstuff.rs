use std::sync::Arc;

use dscale::{Distributions, Jiffies, LatencyRule, SimulationBuilder, global::kv};
use hotstuff::{B0, ChainedHotstuff, HOTSTUFF_POOL, Node};

fn main() {
    let genesis = Arc::new(Node {
        id: 0,
        parent: None,
        height: 0,
        creator: 0,
        creation_time: Jiffies(0),
    });

    kv::set::<Arc<Node>>(B0, genesis);
    kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

    let mut sim = SimulationBuilder::default()
        .add_pool::<ChainedHotstuff>(HOTSTUFF_POOL, 53)
        .latency_topology(&[LatencyRule::WithinPool(
            HOTSTUFF_POOL,
            Distributions::Normal {
                mean: Jiffies(50),
                std_dev: Jiffies(10),
                low: Jiffies(20),
                high: Jiffies(80),
            },
        )])
        .seed(123)
        .time_budget(Jiffies(3600_000))
        .simple()
        .build();

    sim.run_full_budget();

    let ordered = kv::get::<(f64, usize)>("avg_latency").1;
    let avg_latency = kv::get::<(f64, usize)>("avg_latency").0;
    println!("ordered: {ordered}, avg_latency: {avg_latency}")
}
