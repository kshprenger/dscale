use std::{fs::File, sync::Mutex};

use dag_based::bullshark::Bullshark;
use dscale::{BandwidthConfig, Distributions, Jiffies, LatencyRule, SimulationBuilder, global::kv};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let k_validators = 1000;
    let mb_per_sec = [8000, 9000, 10000, 11000];

    mb_per_sec.into_iter().for_each(|bandwidth| {
        let file = Mutex::new(File::create(format!("bullshark_{}.csv", bandwidth)).unwrap());

        let seeds = [4567898765, 33333, 982039];

        seeds.into_par_iter().for_each(|seed| {
            kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

            let mut sim = SimulationBuilder::default()
                .add_pool::<Bullshark>("Validators", k_validators)
                .latency_topology(&[LatencyRule::WithinPool(
                    "Validators",
                    Distributions::Normal {
                        mean: Jiffies(50),
                        std_dev: Jiffies(10),
                        low: Jiffies(20),
                        high: Jiffies(80),
                    },
                )])
                .time_budget(Jiffies(60_000)) // Simulating 1 min of real time execution
                .vnic_bandwidth(BandwidthConfig::Bounded(
                    bandwidth * 1024 * 1024 / (8 * 1000), // bandwidth Mb/sec NICs
                ))
                .seed(seed)
                .simple()
                .build();

            // (avg_latency, total_vertex)
            kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.run_full_budget();

            let ordered = kv::get::<(f64, usize)>("avg_latency").1;
            let avg_latency = kv::get::<(f64, usize)>("avg_latency").0;
            let load = kv::get::<usize>("avg_network_load"); // Bytes per jiffy at single NIC

            writeln!(file.lock().unwrap(), "{} {} {}", ordered, avg_latency, load).unwrap();
        });
    });
}
