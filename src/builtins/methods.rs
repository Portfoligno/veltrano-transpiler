//! Builtin method registration
//!
//! This module handles registration of builtin methods.

use super::types::{BuiltinMethodKind, OperatorMethod};
use std::collections::HashMap;

/// Register all built-in methods
pub fn register_builtin_methods() -> HashMap<String, Vec<BuiltinMethodKind>> {
    let mut methods = HashMap::new();

    // All regular methods have been migrated to the import system:
    // - clone, toString are handled as built-in imports
    // - length is an alias for various .len() methods
    // - toSlice is an alias for Vec::as_slice

    // Only operator methods remain - these generate operators instead of function calls
    let operators = vec![
        OperatorMethod::Ref,
        OperatorMethod::MutRef,
        OperatorMethod::BumpRef,
    ];

    for op in operators {
        methods
            .entry(op.method_name().to_string())
            .or_insert_with(Vec::new)
            .push(BuiltinMethodKind::Operator(op));
    }

    methods
}
