use std::sync::atomic::{AtomicUsize, Ordering};

use dscale::*;

pub static STEPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct HeavyMessage {
    pub nonce: u64,
}

impl Message for HeavyMessage {}

#[derive(Default)]
pub struct HeavyProcess {}

/// Simulate CPU-heavy work per step (e.g. signature verification, hashing).
fn busy_work(nonce: u64) -> u64 {
    let mut hash = nonce;
    for _ in 0..5_000 {
        hash = std::hint::black_box(hash.wrapping_mul(6364136223846793005).wrapping_add(1));
    }
    hash
}

impl ProcessHandle for HeavyProcess {
    fn on_start(&self) {
        schedule_timer_after(Jiffies(100));
    }

    fn on_message(&self, _from: Rank, message: MessagePtr) {
        let msg = message.as_type::<HeavyMessage>();
        let _ = busy_work(msg.nonce);
        STEPS.fetch_add(1, Ordering::Relaxed);
    }

    fn on_timer(&self, _id: TimerId) {
        let nonce = busy_work(rank() as u64);
        broadcast(HeavyMessage { nonce });
        schedule_timer_after(Jiffies(100));
    }
}
