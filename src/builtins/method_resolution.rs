//! Complex method resolution logic
//!
//! This module contains all the complex logic for resolving method calls,
//! including trait checking, ownership transformations, and return type computation.

use super::types::{BuiltinMethodKind, OperatorMethod};
use crate::rust_interop::{RustInteropRegistry, SelfKind};
use crate::types::{TypeConstructor, VeltranoType};
use std::collections::HashMap;

/// Get the return type for a method call (with trait checking)
/// This checks both built-in methods and imported methods
pub fn get_method_return_type(
    method_name: &str,
    receiver_type: &VeltranoType,
    methods: &HashMap<String, Vec<BuiltinMethodKind>>,
    trait_checker: &mut RustInteropRegistry,
) -> Option<VeltranoType> {
    // First check built-in operator methods
    if let Some(method_variants) = methods.get(method_name) {
        // For operator methods, we just need to compute the return type
        // They apply to all types
        if let Some(BuiltinMethodKind::Operator(op)) = method_variants.first() {
            return Some(compute_operator_return_type(op, receiver_type));
        }
    }

    // If not found in built-ins, check imported methods
    get_imported_method_return_type(method_name, receiver_type, trait_checker)
}

/// Check if a Veltrano receiver type can provide the required Rust access for imported methods
/// This is similar to receiver_can_provide_rust_access but doesn't require trait checking
/// since we already know the method exists from the imported signature
pub fn receiver_can_provide_rust_access_for_imported(
    receiver_type: &VeltranoType,
    rust_self_kind: &SelfKind,
    _trait_checker: &mut RustInteropRegistry,
) -> bool {
    match rust_self_kind {
        SelfKind::Ref(_) => {
            // Rust method takes &self - ONLY Ref<T> and naturally referenced types can provide this
            match &receiver_type.constructor {
                TypeConstructor::Ref => true,
                TypeConstructor::Own => false, // No auto-borrow from Own<T>
                _ => true, // Naturally referenced types (String, etc.) can provide &self
            }
        }
        SelfKind::MutRef(_) => {
            // Rust method takes &mut self - only MutRef<T> can provide this
            matches!(&receiver_type.constructor, TypeConstructor::MutRef)
        }
        SelfKind::Value => {
            // Rust method takes self (consumes the value) - only owned types work
            match &receiver_type.constructor {
                TypeConstructor::Own => true,
                _ => true, // Naturally owned types (I64, Bool, etc.) can be consumed
            }
        }
        SelfKind::None => {
            // Associated function - no receiver check needed
            true
        }
    }
}

/// Compute the return type for operator methods
fn compute_operator_return_type(op: &OperatorMethod, receiver_type: &VeltranoType) -> VeltranoType {
    match op {
        OperatorMethod::Ref | OperatorMethod::BumpRef => {
            // Implement correct ref() semantics:
            // Own<T> → T, T → Ref<T>
            match &receiver_type.constructor {
                // Own<T> → T (remove the Own wrapper)
                TypeConstructor::Own => {
                    if let Some(inner) = receiver_type.inner() {
                        inner.clone()
                    } else {
                        // Shouldn't happen with well-formed Own<T>
                        VeltranoType::ref_(receiver_type.clone())
                    }
                }
                // T → Ref<T> (add a Ref wrapper)
                _ => VeltranoType::ref_(receiver_type.clone()),
            }
        }
        OperatorMethod::MutRef => VeltranoType::mut_ref(receiver_type.clone()),
    }
}

/// Get return type for an imported method
fn get_imported_method_return_type(
    method_name: &str,
    receiver_type: &VeltranoType,
    trait_checker: &mut RustInteropRegistry,
) -> Option<VeltranoType> {
    // Use same lookup logic as is_imported_method_available
    // Get the appropriate type for method lookup
    let rust_type = receiver_type.to_rust_type(trait_checker);

    if let Ok(Some(method_info)) = trait_checker.query_method_signature(&rust_type, method_name) {
        // Check if the receiver can provide the required access
        if receiver_can_provide_rust_access_for_imported(
            receiver_type,
            &method_info.self_kind,
            trait_checker,
        ) {
            // TEMPORARY FIX: Special handling for clone method
            // TODO: This is a temporary workaround until we have a more systematic way
            // to handle method return type transformations based on Veltrano semantics
            if method_name == "clone" {
                // Clone has special semantics in Veltrano:
                // - Ref<T>.clone() -> T (not Own<T>)
                // - T.clone() -> Own<T> (for naturally referenced types)
                match &receiver_type.constructor {
                    TypeConstructor::Ref | TypeConstructor::MutRef => {
                        // For Ref<T> or MutRef<T>, clone returns T directly
                        if let Some(inner) = receiver_type.inner() {
                            return Some(inner.clone());
                        }
                    }
                    _ => {
                        // For other types, use normal conversion
                        if let Ok(veltrano_return_type) = method_info.return_type.to_veltrano_type()
                        {
                            return Some(veltrano_return_type);
                        }
                    }
                }
            }

            // For non-clone methods, use normal conversion
            if let Ok(veltrano_return_type) = method_info.return_type.to_veltrano_type() {
                return Some(veltrano_return_type);
            }
        }
    }

    None
}
