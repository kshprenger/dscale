use rand::{Rng, SeedableRng, distr::Uniform};

pub type Seed = u64;

pub struct Randomizer {
    rnd: rand::rngs::StdRng,
}

impl Randomizer {
    pub fn new(seed: Seed) -> Self {
        Self {
            rnd: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    pub fn random_from_range_uniform(&mut self, lower_bound: usize, upper_bound: usize) -> usize {
        let uniform = Uniform::new_inclusive(lower_bound, upper_bound).expect("Invalid bounds");
        self.rnd.sample(uniform)
    }
}
