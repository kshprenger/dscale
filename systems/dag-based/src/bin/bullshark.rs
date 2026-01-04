use std::time::Instant;

use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};
use rayon::prelude::*;

fn main() {
    // let shard = env::var("SHARD").unwrap().parse::<usize>();
    let start = Instant::now();
    (4..200).into_par_iter().for_each(|proc_num| {
        metrics::Clear();
        metrics::Set::<Vec<Jiffies>>("latency", Vec::new());
        metrics::Set::<usize>("timeouts-fired", 0);
        SimulationBuilder::NewDefault()
            .AddPool("Validators", proc_num, || Bullshark::New())
            .MaxLatency(Jiffies(600))
            .TimeBudget(Jiffies(60 * 1000))
            .NICBandwidth(BandwidthType::Unbounded)
            .Seed(proc_num as u64)
            .Build()
            .Run();
        println!(
            "{proc_num}: Ordered: {}",
            metrics::Get::<Vec<Jiffies>>("latency").len()
        );
    });
    println!("elapsed: {:?}", start.elapsed())
}
