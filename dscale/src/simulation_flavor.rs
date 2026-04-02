use crate::runners::threads::Threads;

#[derive(Default)]
pub(crate) enum SimulationFlavor {
    #[default]
    Simple,
    Parallel(Threads),
}
