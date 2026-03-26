pub(crate) mod clock;
/// Per-process configuration (seed, total process count).
pub mod configuration;
/// Thread-safe key-value store shared across all processes.
pub mod kv;
pub(crate) mod local_access;
mod shared_access;
mod tso;

pub(crate) use clock::fast_forward_clock;
pub use clock::now;

pub(crate) fn reset() {
    clock::reset();
    tso::reset();
    shared_access::reset();
    local_access::reset();
    kv::reset();
}
pub use local_access::broadcast;
pub use local_access::broadcast_within_pool;
pub use local_access::choose_from_pool;
pub use local_access::rank;
pub use local_access::schedule_timer_after;
pub use local_access::send_random;
pub use local_access::send_random_from_pool;
pub use local_access::send_to;
pub use shared_access::list_pool;
pub(crate) use shared_access::setup_shared_access;
pub use tso::global_unique_id;
