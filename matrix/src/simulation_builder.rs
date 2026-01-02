use crate::{
    Simulation, network::BandwidthType, process::ProcessHandle, random::Seed, time::Jiffies,
};

fn InitLogger() {
    let _ = env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let module_path = record.module_path().unwrap_or("unknown");
            let crate_name = module_path.split("::").next().unwrap_or(module_path);
            use std::io::Write;
            writeln!(buf, "[{}] {}", crate_name, record.args())
        })
        .try_init();
}

pub struct SimulationBuilder<F>
where
    F: Fn() -> Box<dyn ProcessHandle>,
{
    seed: Seed,
    time_budget: Jiffies,
    max_network_latency: Jiffies,
    process_count: usize,
    factory: F,
    bandwidth: BandwidthType,
}

impl<F> SimulationBuilder<F>
where
    F: Fn() -> Box<dyn ProcessHandle>,
{
    pub fn NewFromFactory(f: F) -> SimulationBuilder<F> {
        SimulationBuilder {
            seed: 69,
            time_budget: Jiffies(1_000_000),
            max_network_latency: Jiffies(10),
            process_count: 1,
            factory: f,
            bandwidth: BandwidthType::Unbounded,
        }
    }

    pub fn Seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    pub fn TimeBudget(mut self, time_budget: Jiffies) -> Self {
        self.time_budget = time_budget;
        self
    }

    pub fn MaxLatency(mut self, max_network_latency: Jiffies) -> Self {
        self.max_network_latency = max_network_latency;
        self
    }

    pub fn ProcessInstances(mut self, count: usize) -> Self {
        self.process_count = count;
        self
    }

    pub fn NICBandwidth(mut self, bandwidth: BandwidthType) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn Build(self) -> Simulation {
        InitLogger();

        Simulation::New(
            self.seed,
            self.time_budget,
            self.max_network_latency,
            self.bandwidth,
            (1..=self.process_count)
                .map(|id| (id, (self.factory)()))
                .collect(),
        )
    }
}
