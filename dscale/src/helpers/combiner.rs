/// Collects values until a threshold is reached, then yields them all at once.
/// Useful for implementing quorum-based logic.
pub struct Combiner<T: Sized> {
    values: Vec<T>,
    threshold: usize,
    idx: usize,
}

impl<T: Sized> Combiner<T> {
    /// Creates a new combiner that fires after `threshold` values.
    pub fn new(threshold: usize) -> Self {
        debug_assert!(
            threshold > 0,
            "Combinter threshold should be greater than zero"
        );
        Self {
            values: Vec::with_capacity(threshold),
            threshold,
            idx: 0,
        }
    }

    /// Adds a value. Returns `Some(all_values)` when the threshold is reached,
    /// `None` otherwise. After firing, subsequent calls return `None`.
    pub fn combine(&mut self, value: T) -> Option<Vec<T>> {
        if self.idx >= self.threshold {
            return None;
        }

        self.values.push(value);
        self.idx += 1;

        if self.idx == self.threshold {
            Some(std::mem::take(&mut self.values))
        } else {
            None
        }
    }
}
