use crate::{random::Randomizer, time::Jiffies};

pub(crate) struct Latency {
    randomizer: Randomizer,
    max_latency: Jiffies,
}

impl Latency {
    pub(crate) fn new(randomizer: Randomizer, max_latency: Jiffies) -> Self {
        Self {
            randomizer,
            max_latency,
        }
    }

    pub(crate) fn introduce_random_latency(&mut self) -> Jiffies {
        let random_time = self
            .randomizer
            .random_from_range_uniform(0, self.max_latency.0);
        Jiffies(random_time)
    }
}
