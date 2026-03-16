#[derive(Default)]
pub(crate) enum SimulationFlavor {
    #[default]
    Simple,
    Parallel(usize),
}
