use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() {
    metrics::Set::<Vec<Jiffies>>("latency", Vec::new());
    metrics::Set::<usize>("timeouts-fired", 0);

    (4..10000).into_par_iter().for_each(|proc_num| {
        SimulationBuilder::NewFromFactory(|| Box::new(Bullshark::New()))
            .MaxLatency(Jiffies(50))
            .TimeBudget(Jiffies(10000))
            .NICBandwidth(BandwidthType::Unbounded)
            .ProcessInstances(proc_num)
            .Seed(234565432345)
            .Build()
            .Run();
        println!("{proc_num}")
    });

    println!(
        "Vertices ordered: {}",
        metrics::Get::<Vec<Jiffies>>("latency").unwrap().len()
    );
    println!(
        "Latency distribution: {:?}",
        metrics::Get::<Vec<Jiffies>>("latency").unwrap()
    );
    println!(
        "Timeouts fired: {}",
        metrics::Get::<usize>("timeouts-fired").unwrap()
    );
}
