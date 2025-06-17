//! Utility functions for code generation
//!
//! This module contains common utilities used across the code generator:
//! - Indentation helpers
//! - Rust keyword detection
//! - String formatting utilities

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
