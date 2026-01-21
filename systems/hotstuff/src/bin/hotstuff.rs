use std::rc::Rc;

use dscale::{Distributions, Jiffies, LatencyDescription, SimulationBuilder, global::anykv};
use hotstuff::{B0, ChainedHotstuff, HOTSTUFF_POOL, Node};

fn main() {
    let genesis = Rc::new(Node {
        id: 0,
        parent: None,
        height: 0,
        creator: 0,
        creation_time: Jiffies(0),
    });

    anykv::set::<Rc<Node>>(B0, genesis);
    anykv::set::<(f64, usize)>("avg_latency", (0.0, 0));

    let mut sim = SimulationBuilder::default()
        .add_pool::<ChainedHotstuff>(HOTSTUFF_POOL, 53)
        .latency_topology(&[LatencyDescription::WithinPool(
            HOTSTUFF_POOL,
            Distributions::Normal(Jiffies(50), Jiffies(10)),
        )])
        .seed(123)
        .time_budget(Jiffies(3600_000))
        .build();

    sim.run();

    let ordered = anykv::get::<(f64, usize)>("avg_latency").1;
    let avg_latency = anykv::get::<(f64, usize)>("avg_latency").0;
    println!("ordered: {ordered}, avg_latency: {avg_latency}")
}
