use crate::random::Seed;

pub type ProcessId = usize;

pub struct Configuration {
    pub seed: Seed,
    pub proc_num: usize,
}
