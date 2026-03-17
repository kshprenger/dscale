#[derive(Default)]
pub enum SimulationFlavor {
    #[default]
    Deterministic,
    Parallel(usize),
}
