use std::sync::Mutex;
use std::usize;

pub struct Combiner<T: Sized> {
    inner: Mutex<CombinerInner<T>>,
}

struct CombinerInner<T> {
    values: Vec<T>,
    threshold: usize,
    idx: usize,
}

impl<T: Sized> Combiner<T> {
    pub fn new(threshold: usize) -> Self {
        debug_assert!(
            threshold > 0,
            "Combinter threshold should be greater than zero"
        );
        Self {
            inner: Mutex::new(CombinerInner {
                values: Vec::with_capacity(threshold),
                threshold,
                idx: 0,
            }),
        }
    }

    pub fn combine(&self, value: T) -> Option<Vec<T>> {
        let mut inner = self.inner.lock().unwrap();

        if inner.idx >= inner.threshold {
            return None;
        }

        inner.values.push(value);
        inner.idx += 1;

        if inner.idx == inner.threshold {
            Some(std::mem::take(&mut inner.values))
        } else {
            None
        }
    }
}
