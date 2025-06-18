//! Registry implementations for managing Rust interop items.
//!
//! Provides static and dynamic registries for type information.

mod dynamic_registry;
mod static_registry;

pub use dynamic_registry::DynamicRustRegistry;
pub use static_registry::RustInteropRegistry;
