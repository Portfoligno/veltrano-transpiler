//! Centralized built-in functions and methods for Veltrano
//!
//! This module consolidates all built-in function definitions that were previously
//! scattered across codegen.rs, rust_interop.rs, and implicit in type checking

// Type definitions module
pub mod types;
// Function registration module
mod functions;
// Method registration module
mod methods;
// Method resolution logic
mod method_resolution;

// Re-export all types for convenience
pub use types::*;

use crate::rust_interop::{RustInteropRegistry, SelfKind};
use crate::types::{FunctionSignature, VeltranoType};
use std::collections::HashMap;

/// Registry for all built-in functions and methods
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunctionKind>,
    methods: HashMap<String, Vec<BuiltinMethodKind>>, // Method name -> list of variants
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        Self {
            functions: functions::register_builtin_functions(),
            methods: methods::register_builtin_methods(),
        }
    }



    /// Check if a function is a Rust macro (skips type checking)
    pub fn is_rust_macro(&self, name: &str) -> bool {
        functions::is_rust_macro(name, &self.functions)
    }

    /// Get function signatures for type checker initialization
    pub fn get_function_signatures(&self) -> Vec<FunctionSignature> {
        functions::get_function_signatures(&self.functions)
    }

    /// Get the return type for a method call (with trait checking)
    /// This checks both built-in methods and imported methods
    pub fn get_method_return_type(
        &self,
        method_name: &str,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Option<VeltranoType> {
        method_resolution::get_method_return_type(
            method_name,
            receiver_type,
            &self.methods,
            trait_checker,
        )
    }


    /// Check if a Veltrano receiver type can provide the required Rust access for imported methods
    /// This is similar to receiver_can_provide_rust_access but doesn't require trait checking
    /// since we already know the method exists from the imported signature
    pub fn receiver_can_provide_rust_access_for_imported(
        &self,
        receiver_type: &VeltranoType,
        rust_self_kind: &SelfKind,
        trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        method_resolution::receiver_can_provide_rust_access_for_imported(
            receiver_type,
            rust_self_kind,
            trait_checker,
        )
    }

}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
