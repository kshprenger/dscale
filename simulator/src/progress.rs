use std::env;

use indicatif::{ProgressBar, ProgressStyle};
use log::Level;

use crate::Jiffies;

pub(crate) struct Bar {
    bar: ProgressBar,
    prev_log: usize,
    k: usize,
}

impl Bar {
    pub(crate) fn New(total: Jiffies, k: usize) -> Self {
        let bar = ProgressBar::new(total.0 as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:100.cyan/blue}] {pos}/{len} Jiffies {msg}")
                .unwrap(),
        );
        Self {
            bar: bar,
            prev_log: 0,
            k,
        }
    }

    pub(crate) fn MakeProgress(&mut self, time: Jiffies) {
        let d = time.0 / self.k;
        if d > self.prev_log {
            self.prev_log = d;
            match env::var("RUST_LOG") {
                Ok(value) => {
                    if value == "info" {
                        self.bar.set_position(time.0 as u64)
                    }
                }
                Err(_) => {}
            }
        }
    }
}
