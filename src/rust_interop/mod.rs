//! Module for handling Rust interoperability and type signature extraction
//! This provides mechanisms to:
//! 1. Declare external Rust functions/methods
//! 2. Parse Rust type signatures
//! 3. Convert between Rust and Veltrano type representations
//! 4. Dynamically query Rust toolchain for type information

mod cache;
mod parser;
mod registry;
mod rustdoc_querier;
mod stdlib_querier;
mod syn_querier;
mod types;
mod utils;

pub use cache::{CrateInfo, TypeInfo};

// Re-export for tests only. Not part of the stable public API.
#[doc(hidden)]
#[allow(unused_imports)]
pub use cache::{MethodInfo, RustTypeSignature, TypeKind};
pub use parser::RustTypeParser;
/// Exposed for testing only. Not part of the stable public API.
#[doc(hidden)]
#[allow(unused_imports)]
pub use registry::DynamicRustRegistry;
pub use registry::RustInteropRegistry;
/// Exposed for testing only. Not part of the stable public API.
#[doc(hidden)]
#[allow(unused_imports)]
pub use rustdoc_querier::RustdocQuerier;
#[doc(hidden)]
#[allow(unused_imports)]
pub use stdlib_querier::StdLibQuerier;
/// Exposed for testing only. Not part of the stable public API.
#[doc(hidden)]
#[allow(unused_imports)]
pub use syn_querier::SynQuerier;
pub use types::{RustType, SelfKind};
pub use utils::camel_to_snake_case;

use crate::error::VeltranoError;

/// Represents an external Rust item (function, method, or type)
#[derive(Debug, Clone)]
pub enum ExternItem {
    Function {
        name: String,
        _path: String, // Full Rust path e.g., "std::vec::Vec::new"
        _params: Vec<(String, RustType)>,
        _return_type: RustType,
        _is_unsafe: bool,
    },
    Method {
        type_name: String,
        method_name: String,
        _self_kind: SelfKind,
        _params: Vec<(String, RustType)>,
        _return_type: RustType,
        _is_unsafe: bool,
    },
    _Type {
        name: String,
        rust_path: String,
        generic_params: Vec<String>,
    },
}

// === Dynamic Rust Interop System ===

/// Error types for dynamic Rust interop operations
#[derive(Debug, Clone)]
pub enum RustInteropError {
    CargoError(String),
    ParseError(String),
    IoError(String),
    CrateNotFound(String),
}

impl std::fmt::Display for RustInteropError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RustInteropError::CargoError(msg) => write!(f, "Cargo error: {}", msg),
            RustInteropError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            RustInteropError::IoError(msg) => write!(f, "IO error: {}", msg),
            RustInteropError::CrateNotFound(name) => write!(f, "Crate not found: {}", name),
        }
    }
}

impl std::error::Error for RustInteropError {}

/// Trait for querying Rust type information
pub trait RustQuerier: std::fmt::Debug {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, VeltranoError>;
    fn supports_crate(&self, crate_name: &str) -> bool;
    fn priority(&self) -> u32; // Higher priority queriers tried first
}
