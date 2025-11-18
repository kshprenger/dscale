use crate::{
    communication::{Event, EventQueue},
    metrics::{self, Metrics},
    process::Process,
    random::{self, Randomizer},
};

pub struct Simulation {
    randomizer: Randomizer,
    nodes: Vec<(Box<dyn Process>, EventQueue)>,
    metrics: Metrics,
    rt: tokio::runtime::Runtime,
}

impl Simulation {
    pub(crate) fn new(seed: random::Seed) -> Self {
        Self {
            randomizer: Randomizer::new(seed),
            nodes: Vec::new(),
            metrics: Metrics {},
            rt: tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap(),
        }
    }

    pub(crate) fn submit_event(&mut self, event: Event) {}

    pub(crate) fn add_process(&mut self, process: impl Process) {}

    pub(crate) fn run(&mut self) {}

    pub(crate) fn stop(&mut self) -> metrics::Metrics {
        Metrics {}
    }
}
