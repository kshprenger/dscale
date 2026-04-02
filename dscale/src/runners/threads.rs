/// Represents how many threads will workers use to run steps in parallel.
pub enum Threads {
    /// Thread number that will match number of cores on your machine.
    All,
    /// Specific number of threads.
    Specific(usize),
}

impl Into<usize> for Threads {
    fn into(self) -> usize {
        match self {
            Threads::All => std::thread::available_parallelism()
                .expect("could not acquire machine core number")
                .get(),
            Threads::Specific(number) => number,
        }
    }
}
