use std::{fs::File, sync::Mutex};

use dag_based::sparse_bullshark::SparseBullshark;
use dscale::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::kv,
    time::Jiffies,
};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let k_validators = 1000;
    let mb_per_sec = [8000, 9000, 10000, 11000];

    mb_per_sec.into_iter().for_each(|bandwidth| {
        let file = Mutex::new(File::create(format!("sparse_bullshark_{}.csv", bandwidth)).unwrap());

        let seeds = [4567898765, 33333, 982039];
        // 5% to quorum by 5 % step
        let samples = (((k_validators as f64 * 0.05) as usize)
            ..=((k_validators as f64 * 0.66) as usize))
            .step_by((k_validators as f64 * 0.05) as usize);
        let product = samples.flat_map(|x| seeds.iter().map(move |y| (x, y)));

        product.par_bridge().into_par_iter().for_each(|(d, seed)| {
            kv::set::<(f64, usize)>("avg_latency", (0.0, 0));
            kv::set::<(f64, usize)>("avg_virtual_size", (0.0, 0));
            kv::set::<usize>("D", d); // Sample size

            let mut sim = SimulationBuilder::default()
                .add_pool::<SparseBullshark>("Validators", k_validators)
                .latency_topology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .time_budget(Jiffies(60_000)) // Simulating 1 min of real time execution
                .nic_bandwidth(BandwidthDescription::Bounded(
                    bandwidth * 1024 * 1024 / (8 * 1000), // bandwidth Mb/sec NICs
                ))
                .seed(*seed)
                .build();

            // (avg_latency, total_vertex)
            kv::set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.run();

            let ordered = kv::get::<(f64, usize)>("avg_latency").1;
            let avg_latency = kv::get::<(f64, usize)>("avg_latency").0;
            let load = kv::get::<usize>("avg_network_load"); // Bytes per jiffy at single NIC
            let avg_virtual_size_of_message = kv::get::<(f64, usize)>("avg_virtual_size");

            writeln!(
                file.lock().unwrap(),
                "{} {} {} {} {}",
                d,
                ordered,
                avg_latency,
                load,
                avg_virtual_size_of_message.0,
            )
            .unwrap();
        });
    });
}
