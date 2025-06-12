/// Debug output control for the Veltrano transpiler
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to control debug output
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable debug output
pub fn enable_debug() {
    DEBUG_ENABLED.store(true, Ordering::Relaxed);
}

/// Check if debug output is enabled
pub fn is_debug_enabled() -> bool {
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
