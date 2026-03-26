/// Logs a debug message prefixed with the current simulation time and process rank.
/// Controlled by the `RUST_LOG` environment variable.
#[macro_export]
macro_rules! debug_process {
    ($($arg:tt)+) => {
        log::debug!("[Now: {} | P{}] {}", now(), rank(), format_args!($($arg)+));
    }
}
