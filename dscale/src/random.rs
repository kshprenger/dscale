use rand::{Rng, SeedableRng, distr::Uniform, seq::IndexedRandom};
use rand_distr::{Bernoulli, Normal};

use crate::Jiffies;

pub type Seed = u64;

#[derive(Copy, Clone, Debug)]
pub enum Distributions {
    Uniform(Jiffies, Jiffies),
    Bernoulli(f64, Jiffies),
    Normal(Jiffies, Jiffies),
}

impl Distributions {
    pub(super) fn safe_window(&self) -> Jiffies {
        match self.clone() {
            Self::Uniform(l, _) => l,
            Self::Bernoulli(_, _) => Jiffies(1),
            Self::Normal(a, b) => a - b,
        }
    }
}

#[derive(Debug)]
pub struct Randomizer {
    rnd: rand::rngs::SmallRng,
}

impl Default for Randomizer {
    fn default() -> Self {
        Self {
            rnd: rand::rngs::SmallRng::seed_from_u64(0),
        }
    }
}

impl Randomizer {
    pub fn new(seed: Seed) -> Self {
        Self {
            rnd: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }

    pub fn random_usize(&mut self, d: Distributions) -> usize {
        match d {
            Distributions::Uniform(Jiffies(from), Jiffies(to)) => {
                let distr = Uniform::new_inclusive(from, to).expect("Invalid bounds");
                self.rnd.sample(distr)
            }
            Distributions::Bernoulli(p, Jiffies(val)) => {
                let distr = Bernoulli::new(p).expect("Invalid probability");
                if self.rnd.sample(distr) { val } else { 0 }
            }
            Distributions::Normal(Jiffies(mean), Jiffies(std_dev)) => {
                let distr = Normal::new(mean as f64, std_dev as f64).expect("Invalid parameters");
                self.rnd.sample(distr).max(0.0).round() as usize
            }
        }
    }

    pub fn choose_from_slice<'a, T: Copy>(&mut self, from: &[T]) -> T {
        from.choose(&mut self.rnd)
            .copied()
            .expect("Chose from empty slice")
    }
}
