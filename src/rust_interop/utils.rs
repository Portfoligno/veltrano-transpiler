//! Utility functions for Rust interop
//!
//! This module contains helper functions used across the rust_interop module.

/// Convert camelCase to snake_case for Rust naming conventions
pub fn camel_to_snake_case(name: &str) -> String {
    let mut result = String::new();

    for ch in name.chars() {
        if ch == '_' {
            // Underscore becomes double underscore
            result.push_str("__");
        } else if ch.is_uppercase() {
            // Uppercase becomes underscore + lowercase
            result.push('_');
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            // Lowercase stays as is
            result.push(ch);
        }
    }

    result
}
