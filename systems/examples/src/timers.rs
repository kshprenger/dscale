use std::sync::atomic::{AtomicUsize, Ordering};

use dscale::{global::kv, *};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum LazyPingPongMessage {
    Ping,
    DelayedPong,
}

impl Message for LazyPingPongMessage {}

/// A sentinel value indicating no timer is set.
const NO_TIMER: usize = usize::MAX;

pub struct LazyPingPong {
    heartbeat_timer: AtomicUsize,
    ping_count: AtomicUsize,
}

impl Default for LazyPingPong {
    fn default() -> Self {
        Self {
            heartbeat_timer: AtomicUsize::new(NO_TIMER),
            ping_count: AtomicUsize::new(0),
        }
    }
}

impl ProcessHandle for LazyPingPong {
    fn start(&self) {
        debug_process!("Starting timer demo process");

        // Schedule a heartbeat timer to fire every 1000 jiffies
        let timer_id = schedule_timer_after(Jiffies(1000));
        self.heartbeat_timer.store(timer_id, Ordering::Relaxed);
        debug_process!(
            "Scheduled heartbeat timer {} to fire in 1000 jiffies",
            timer_id
        );

        // Process 0 starts by sending a ping
        if rank() == 0 {
            send_to(1, LazyPingPongMessage::Ping);
        }
    }

    fn on_message(&self, from: Rank, message: MessagePtr) {
        let m = message.as_type::<LazyPingPongMessage>();

        match m {
            LazyPingPongMessage::Ping => {
                debug_process!("Received Ping from Process {}", from);
                kv::modify::<usize>("pings_received", |count| *count += 1);

                // Schedule a delayed response using a timer
                let timer_id = schedule_timer_after(Jiffies(500));
                debug_process!("Scheduling delayed pong response with timer {}", timer_id);
            }

            LazyPingPongMessage::DelayedPong => {
                debug_process!("Received DelayedPong from Process {}", from);
                kv::modify::<usize>("pongs_received", |count| *count += 1);

                // Send another ping if we haven't reached the limit
                let count = self.ping_count.fetch_add(1, Ordering::Relaxed);
                if count < 5 {
                    send_to(from, LazyPingPongMessage::Ping);
                }
            }
        }
    }

    fn on_timer(&self, timer_id: TimerId) {
        debug_process!("Timer {} fired", timer_id);

        // Check if this is the heartbeat timer
        let heartbeat_id = self.heartbeat_timer.load(Ordering::Relaxed);
        if heartbeat_id != NO_TIMER && timer_id == heartbeat_id {
            debug_process!("Heartbeat timer fired");
            kv::modify::<usize>("heartbeats", |count| *count += 1);

            // Reschedule the heartbeat timer for continuous operation
            let new_timer_id = schedule_timer_after(Jiffies(1000));
            self.heartbeat_timer.store(new_timer_id, Ordering::Relaxed);
            return;
        }

        // This must be a delayed response timer
        debug_process!("Delayed response timer fired - sending DelayedPong");
        if rank() == 1 {
            send_to(0, LazyPingPongMessage::DelayedPong);
        }
    }
}
