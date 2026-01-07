use std::fs::File;

use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, global::anykv, time::Jiffies};
use rayon::prelude::*;

use std::io::Write;

fn main() {
    let mut file = File::create("results.csv").unwrap();

    writeln!(file, "validators, ordered, avg_latency").unwrap();

    (4..600)
        .into_par_iter()
        .map(|proc_num| {
            // 1 jiffy == 1 real millisecond
            let sim = SimulationBuilder::NewDefault()
                .AddPool("Validators", proc_num, Bullshark::New)
                .MaxLatency(Jiffies(400)) // 400 ms of max network latency
                .TimeBudget(Jiffies(60_000)) // Simulating 1 min of real time execution
                .NICBandwidth(BandwidthType::Bounded(1 * 1024 * 1024 * 1024 / (8 * 1000))) // 10Gb/sec NICs
                .Seed(proc_num as u64)
                .Build();

            anykv::Set::<Vec<Jiffies>>("latency", Vec::new());
            anykv::Set::<usize>("timeouts-fired", 0);

            sim.Run();
            println!("Simulation done for {proc_num} validators");

            let ordered = anykv::Get::<Vec<Jiffies>>("latency").len();
            let average_latency = anykv::Get::<Vec<Jiffies>>("latency")
                .iter()
                .map(|&x| x.0 as f64)
                .enumerate()
                .fold(0.0, |acc, (i, x)| acc + (x - acc) / (i + 1) as f64);

            anykv::Clear();

            (proc_num, ordered, average_latency)
        })
        .collect::<Vec<(usize, usize, f64)>>()
        .into_iter()
        .for_each(|(proc_mum, ordered, average_latency)| {
            writeln!(file, "{} {} {}", proc_mum, ordered, average_latency).unwrap();
        });
}
