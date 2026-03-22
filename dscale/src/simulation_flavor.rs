#[derive(Default)]
pub(crate) enum SimulationFlavor {
    #[default]
    Deterministic,
    Parallel(usize),
}
