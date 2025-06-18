//! Code generation utilities.
//!
//! Common helpers for indentation and macro detection.

use super::CodeGenerator;

impl CodeGenerator {
    /// Adds the current indentation level to the output
    pub(super) fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    /// Checks if a given name is a Rust macro
    pub(super) fn is_rust_macro(&self, name: &str) -> bool {
        matches!(
            name,
            "println" | "print" | "panic" | "assert" | "debug_assert"
        )
    }
}
