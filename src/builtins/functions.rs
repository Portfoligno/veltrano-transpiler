//! Builtin function registration and queries
//!
//! This module handles registration of builtin functions and provides
//! functionality to query them.

use super::types::BuiltinFunctionKind;
use crate::types::{FunctionSignature, VeltranoType};
use std::collections::HashMap;

/// Register all built-in functions
pub fn register_builtin_functions() -> HashMap<String, BuiltinFunctionKind> {
    let mut functions = HashMap::new();

    // Rust macros (variadic, skip type checking)
    let rust_macros = vec!["println", "print", "panic", "assert", "debug_assert"];
    for macro_name in rust_macros {
        functions.insert(
            macro_name.to_string(),
            BuiltinFunctionKind::RustMacro {
                macro_name: macro_name.to_string(),
            },
        );
    }

    // Special functions with specific signatures
    functions.insert(
        "MutRef".to_string(),
        BuiltinFunctionKind::SpecialFunction {
            function_name: "MutRef".to_string(),
            parameters: vec![VeltranoType::generic(
                "T".to_string(),
                vec!["Clone".to_string()],
            )], // Generic parameter with Clone constraint
            return_type: VeltranoType::mut_ref(VeltranoType::generic(
                "T".to_string(),
                vec!["Clone".to_string()],
            )),
        },
    );

    functions
}

/// Check if a function is a Rust macro (skips type checking)
pub fn is_rust_macro(name: &str, functions: &HashMap<String, BuiltinFunctionKind>) -> bool {
    if let Some(BuiltinFunctionKind::RustMacro { .. }) = functions.get(name) {
        true
    } else {
        false
    }
}

/// Get function signatures for type checker initialization
pub fn get_function_signatures(
    functions: &HashMap<String, BuiltinFunctionKind>,
) -> Vec<FunctionSignature> {
    let mut signatures = Vec::new();

    for (_name, kind) in functions {
        match kind {
            BuiltinFunctionKind::RustMacro { .. } => {
                // Skip macros - they don't participate in type checking
            }
            BuiltinFunctionKind::SpecialFunction {
                function_name,
                parameters,
                return_type,
            } => {
                signatures.push(FunctionSignature {
                    name: function_name.clone(),
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                });
            }
        }
    }

    signatures
}
