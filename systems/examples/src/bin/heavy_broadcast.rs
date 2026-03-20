use std::sync::atomic::Ordering;
use std::time::Instant;

use dscale::*;
use examples::heavy_broadcast::{HeavyProcess, STEPS};

fn main() {
    let num_procs = 200;

    let base_sim = || {
        SimulationBuilder::default()
            .add_pool::<HeavyProcess>("heavy", num_procs)
            .nic_bandwidth(BandwidthDescription::Unbounded)
            .latency_topology(&[LatencyDescription::WithinPool(
                "heavy",
                Distributions::Uniform(Jiffies(100), Jiffies(150)),
            )])
            .time_budget(Jiffies(10_000))
            .seed(42)
    };

    // Deterministic (single-threaded)
    STEPS.store(0, Ordering::Relaxed);
    let mut det = base_sim().deterministic().build();
    let start = Instant::now();
    det.run_full_budget();
    let det_elapsed = start.elapsed();
    let det_steps = STEPS.load(Ordering::Relaxed);
    drop(det);

    println!("Deterministic: {:?}, steps: {}", det_elapsed, det_steps);
    println!(
        "  steps/sec: {:.2}",
        det_steps as f64 / det_elapsed.as_secs_f64()
    );

    // Parallel (7 cores)
    STEPS.store(0, Ordering::Relaxed);
    let mut par = base_sim().parallel(23).build();
    let start = Instant::now();
    par.run_full_budget();
    let par_elapsed = start.elapsed();
    let par_steps = STEPS.load(Ordering::Relaxed);
    drop(par);

    println!(
        "\nParallel (23 cores): {:?}, steps: {}",
        par_elapsed, par_steps
    );
    println!(
        "  steps/sec: {:.2}",
        par_steps as f64 / par_elapsed.as_secs_f64()
    );

    println!(
        "\nSpeedup: {:.2}x",
        det_elapsed.as_secs_f64() / par_elapsed.as_secs_f64()
    );
}
