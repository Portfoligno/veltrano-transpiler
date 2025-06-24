//! Builtin method registration
//!
//! This module handles registration of builtin methods.

use super::types::{BuiltinMethodKind, MethodReturnTypeStrategy, TypeFilter};
use std::collections::HashMap;

/// Register all built-in methods
pub fn register_builtin_methods() -> HashMap<String, Vec<BuiltinMethodKind>> {
    let mut methods = HashMap::new();

    // Helper closure to register a method
    let mut register = |method_name: &str, method_kind: BuiltinMethodKind| {
        methods
            .entry(method_name.to_string())
            .or_insert_with(Vec::new)
            .push(method_kind);
    };

    // Universal trait methods have been migrated to the import system
    // clone, toString are now handled as built-in imports

    // Reference creation methods (available on all appropriate types)
    register(
        "ref",
        BuiltinMethodKind::SpecialMethod {
            method_name: "ref".to_string(),
            receiver_type_filter: TypeFilter::All,
            parameters: vec![],
            return_type_strategy: MethodReturnTypeStrategy::RefSemantics,
        },
    );

    register(
        "mutRef",
        BuiltinMethodKind::SpecialMethod {
            method_name: "mutRef".to_string(),
            receiver_type_filter: TypeFilter::All, // Allow .mutRef() on any type
            parameters: vec![],
            return_type_strategy: MethodReturnTypeStrategy::MutRefToReceiver,
        },
    );

    // length has been migrated to the import system as an alias for various .len() methods
    // toSlice has been migrated to the import system as an alias for Vec::as_slice

    // Bump allocation methods (available on all types)
    register(
        "bumpRef",
        BuiltinMethodKind::SpecialMethod {
            method_name: "bumpRef".to_string(),
            receiver_type_filter: TypeFilter::All,
            parameters: vec![],
            return_type_strategy: MethodReturnTypeStrategy::RefSemantics,
        },
    );

    methods
}
