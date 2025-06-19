//! Builtin method registration
//!
//! This module handles registration of builtin methods.

use super::types::{BuiltinMethodKind, MethodReturnTypeStrategy, TypeFilter};
use crate::types::{TypeConstructor, VeltranoType};
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

    // Universal trait methods - signature information will be looked up dynamically
    register(
        "clone",
        BuiltinMethodKind::TraitMethod {
            method_name: "clone".to_string(),
            required_trait: "Clone".to_string(),
        },
    );

    register(
        "toString",
        BuiltinMethodKind::TraitMethod {
            method_name: "toString".to_string(),
            required_trait: "ToString".to_string(),
        },
    );

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

    // Special methods
    register(
        "toSlice",
        BuiltinMethodKind::SpecialMethod {
            method_name: "toSlice".to_string(),
            receiver_type_filter: TypeFilter::TypeConstructors(vec![TypeConstructor::Vec]),
            parameters: vec![],
            return_type_strategy: MethodReturnTypeStrategy::RefToReceiver,
        },
    );

    // Other common methods
    register(
        "length",
        BuiltinMethodKind::SpecialMethod {
            method_name: "length".to_string(),
            receiver_type_filter: TypeFilter::All, // Available on all types for now
            parameters: vec![],
            return_type_strategy: MethodReturnTypeStrategy::FixedType(VeltranoType::i64()),
        },
    );

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
