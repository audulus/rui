//! Opt-in timing: when `RUI_TIMING` is set, the event loop and rendering print how long their
//! steps take to stderr, as `rui-timing <label> <ms> ms` lines. Without it nothing is measured.

use std::time::Instant;

/// Whether `RUI_TIMING` is set. Never on wasm, which has no clock in `std`.
fn enabled() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var_os("RUI_TIMING").is_some())
    }
    #[cfg(target_arch = "wasm32")]
    false
}

/// Starts timing a step: the current time while timing is on, else `None`, without reading the
/// clock.
pub(crate) fn start() -> Option<Instant> {
    enabled().then(Instant::now)
}

/// Prints the time since `start` under `label`, if timing was on when it started.
pub(crate) fn stop(label: &str, start: Option<Instant>) {
    if let Some(start) = start {
        eprintln!(
            "rui-timing {label} {:.3} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}

/// Runs `f`, timing it under `label`.
pub(crate) fn timed<R>(label: &str, f: impl FnOnce() -> R) -> R {
    let start = start();
    let result = f();
    stop(label, start);
    result
}
