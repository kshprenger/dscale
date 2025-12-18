#[derive(Clone, Default)]
pub struct Metrics {
    pub events_total: usize,
}

impl Metrics {
    pub(crate) fn TrackEvent(&mut self) {
        self.events_total += 1;
    }
}
