use rand::{Rng, SeedableRng, distr::Uniform, seq::IndexedRandom};
use rand_distr::{Bernoulli, Normal};

use crate::Jiffies;

pub(crate) type Seed = u64;

#[derive(Copy, Clone, Debug)]
pub enum Distributions {
    Uniform(Jiffies, Jiffies),
    Bernoulli(f64, Jiffies),

    // https://en.wikipedia.org/wiki/Truncated_normal_distribution
    Normal {
        mean: Jiffies,
        std_dev: Jiffies,
        low: Jiffies,
        high: Jiffies,
    },
}

impl Distributions {
    pub(super) fn safe_window(&self) -> Jiffies {
        match self.clone() {
            Self::Uniform(a, _) => a,
            Self::Bernoulli(_, _) => Jiffies(1),
            Self::Normal { low, .. } => low,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Randomizer {
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
    pub(crate) fn new(seed: Seed) -> Self {
        Self {
            rnd: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }

    pub(crate) fn random_usize(&mut self, d: Distributions) -> usize {
        match d {
            Distributions::Uniform(Jiffies(from), Jiffies(to)) => {
                let distr = Uniform::new_inclusive(from, to).expect("Invalid bounds");
                self.rnd.sample(distr)
            }
            Distributions::Bernoulli(p, Jiffies(val)) => {
                let distr = Bernoulli::new(p).expect("Invalid probability");
                if self.rnd.sample(distr) { val } else { 0 }
            }
            Distributions::Normal {
                mean: Jiffies(mean),
                std_dev: Jiffies(std_dev),
                low: Jiffies(low),
                high: Jiffies(high),
            } => {
                let distr = Normal::new(mean as f64, std_dev as f64).expect("Invalid parameters");
                loop {
                    let sample: f64 = self.rnd.sample(distr);
                    let rounded = sample.round() as isize;
                    if rounded >= low as isize && rounded <= high as isize {
                        return rounded as usize;
                    }
                }
            }
        }
    }

    pub(crate) fn choose_from_slice<'a, T: Copy>(&mut self, from: &[T]) -> T {
        from.choose(&mut self.rnd)
            .copied()
            .expect("Chose from empty slice")
    }
}
