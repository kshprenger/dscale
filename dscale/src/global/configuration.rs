use crate::{Rank, global::kv, random::Seed, rank};

pub(crate) fn setup_global_configuration(proc_num: usize) {
    kv::set::<usize>("proc_num", proc_num)
}

pub(crate) fn setup_local_configuration(id: Rank, base_seed: Seed) {
    // Prevent resonance between procs by changing seed a little bit
    kv::set::<u64>(&format!("seeds/{}", id), base_seed + id as u64)
}

pub fn seed() -> Seed {
    kv::get::<u64>(&format!("seeds/{}", rank()))
}

pub fn process_number() -> usize {
    kv::get::<usize>("proc_num")
}
