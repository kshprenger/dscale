use std::{fs::File, sync::Mutex};

use dag_based::sparse_bullshark::SparseBullshark;
use dscale::{BandwidthConfig, Distributions, Jiffies, SimulationBuilder, global::kv};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let k_validators = 2000;
    let thresholds = [1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];

    thresholds.into_iter().for_each(|threshold| {
        let file = Mutex::new(
            File::create(format!("sparse_bullshark_threshold_{}.csv", threshold)).unwrap(),
        );

        let seeds = [1, 2, 3];
        // 5% -> quorum ; by 5% step
        let samples = (((k_validators as f64 * 0.05) as usize)
            ..=((k_validators as f64 * 0.67) as usize))
            .step_by((k_validators as f64 * 0.05) as usize);
        let product = samples.flat_map(|x| seeds.iter().map(move |y| (x, y)));

        product.par_bridge().into_par_iter().for_each(|(d, seed)| {
            kv::set::<(f64, usize)>("avg_latency", (0.0, 0));
            kv::set::<usize>("D", d); // Sample size
            kv::set::<f64>("threshold", threshold); // xf + 1

            let mut sim = SimulationBuilder::default()
                .add_pool::<SparseBullshark>("Validators", k_validators)
                .within_pool_latency(
                    "Validators",
                    Distributions::Normal {
                        mean: Jiffies(50),
                        std_dev: Jiffies(10),
                        low: Jiffies(20),
                        high: Jiffies(80),
                    },
                )
                .time_budget(Jiffies(36000_000)) // Simulating 10 hours of real time execution
                .vnic_bandwidth(BandwidthConfig::Bounded(5 * 1024 * 1024 / (8 * 1000)))
                .seed(*seed)
                .simple()
                .build();

            // (avg_latency, total_vertex)
            kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.run_full_budget();

            let ordered = kv::get::<(f64, usize)>("avg_latency").1;
            let avg_latency = kv::get::<(f64, usize)>("avg_latency").0;

            writeln!(file.lock().unwrap(), "{} {} {}", d, ordered, avg_latency).unwrap();
        });
    });
}
