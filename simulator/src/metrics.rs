use log::info;

const K_PROGRESS_LOG: usize = 1_000_000;

#[derive(Clone, Default)]
pub struct Metrics {
    pub events_total: usize,
}

impl Metrics {
    pub(crate) fn TrackEvent(&mut self) {
        self.events_total += 1;
        if self.events_total % K_PROGRESS_LOG == 0 {
            info!("Events tracked: {}", self.events_total)
        }
    }
}
