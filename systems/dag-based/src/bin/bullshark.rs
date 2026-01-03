use std::time::Instant;

use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};

fn main() {
    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();
    let start = Instant::now();
    pool.scope(|s| {
        (4..100).into_iter().for_each(|proc_num| {
            s.spawn(move |_| {
                metrics::Clear();
                metrics::Set::<Vec<Jiffies>>("latency", Vec::new());
                metrics::Set::<usize>("timeouts-fired", 0);
                SimulationBuilder::NewFromFactory(|| Box::new(Bullshark::New()))
                    .MaxLatency(Jiffies(10))
                    .TimeBudget(Jiffies(10000))
                    .NICBandwidth(BandwidthType::Unbounded)
                    .ProcessInstances(proc_num)
                    .Seed(23456765)
                    .Build()
                    .Run();
                println!("Ordered: {}", metrics::Get::<Vec<Jiffies>>("latency").len());
            });
        })
    });
    println!("elapsed: {:?}", start.elapsed())
}
