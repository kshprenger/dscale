use dscale::{global::kv, *};
use examples::ring::Ring;

fn main() {
    let mut sim = SimulationBuilder::default()
        .add_pool::<Ring>("RingPool", 100)
        .vnic_bandwidth(BandwidthConfig::Unbounded)
        .within_pool_latency("RingPool", Distributions::Uniform(Jiffies(1), Jiffies(10)))
        .time_budget(Jiffies(1_000_000))
        .simple()
        .seed(42)
        .build();

    kv::set::<usize>("passes", 0);

    let mut total_steps = 0;
    loop {
        let status = sim.run_steps(1000);
        total_steps += status.steps();
        println!(
            "status={:?}, passes={}, total_steps={}",
            status,
            kv::get::<usize>("passes"),
            total_steps,
        );
        match status {
            RunStatus::Completed { .. } => {}
            RunStatus::BudgetExhausted { .. } | RunStatus::NoMoreEvents { .. } => break,
        }
    }
}
