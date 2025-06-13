/// Debug output control for the Veltrano transpiler
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

/// Global flag to control debug output
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Ensures environment variable check happens only once
static INIT: Once = Once::new();

/// Initialize debug state from environment variable
fn init_from_env() {
    INIT.call_once(|| {
        if std::env::var("VELTRANO_DEBUG").is_ok() {
            DEBUG_ENABLED.store(true, Ordering::Relaxed);
        }
    });
}

/// Enable debug output
pub fn enable_debug() {
    DEBUG_ENABLED.store(true, Ordering::Relaxed);
}

/// Check if debug output is enabled
pub fn is_debug_enabled() -> bool {
    init_from_env();
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Print debug message only if debug mode is enabled
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}
