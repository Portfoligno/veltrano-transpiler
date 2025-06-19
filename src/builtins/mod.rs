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

// Re-export all types for convenience
pub use types::*;

use crate::rust_interop::{RustInteropRegistry, SelfKind};
use crate::types::{FunctionSignature, TypeConstructor, VeltranoType};
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
        // First check built-in methods
        if let Some(method_variants) = self.methods.get(method_name) {
            for method_kind in method_variants {
                if self.method_matches_receiver(method_kind, receiver_type, trait_checker) {
                    return Some(self.compute_return_type(
                        method_kind,
                        receiver_type,
                        trait_checker,
                    ));
                }
            }
        }

        // If not found in built-ins, check imported methods
        self.get_imported_method_return_type(method_name, receiver_type, trait_checker)
    }

    /// Check if a Veltrano receiver type can provide the required Rust access
    /// and if the underlying type implements the required trait
    fn receiver_can_provide_rust_access(
        &self,
        receiver_type: &VeltranoType,
        rust_self_kind: &SelfKind,
        required_trait: &str,
        trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        match rust_self_kind {
            SelfKind::Ref => {
                // Rust method takes &self - ONLY Ref<T> can provide this in Veltrano's explicit system
                match &receiver_type.constructor {
                    // Ref<T> can provide &T - check if T implements the trait
                    TypeConstructor::Ref => {
                        if let Some(inner_type) = receiver_type.args.first() {
                            let rust_type = inner_type.to_rust_type(trait_checker);
                            trait_checker
                                .type_implements_trait(&rust_type, required_trait)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    // Own<T> CANNOT auto-borrow - explicit conversion required
                    TypeConstructor::Own => false,
                    // T (naturally referenced types) can provide &T - check if T implements the trait
                    _ => {
                        let rust_type = receiver_type.to_rust_type(trait_checker);
                        trait_checker
                            .type_implements_trait(&rust_type, required_trait)
                            .unwrap_or(false)
                    }
                }
            }
            SelfKind::MutRef => {
                // Rust method takes &mut self - only MutRef<T> can provide this
                match &receiver_type.constructor {
                    TypeConstructor::MutRef => {
                        if let Some(inner_type) = receiver_type.args.first() {
                            let rust_type = inner_type.to_rust_type(trait_checker);
                            trait_checker
                                .type_implements_trait(&rust_type, required_trait)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    _ => false, // Only MutRef<T> can provide &mut access
                }
            }
            SelfKind::Value => {
                // Rust method takes self (consumes the value) - only owned types work
                match &receiver_type.constructor {
                    TypeConstructor::Own => {
                        if let Some(inner_type) = receiver_type.args.first() {
                            let rust_type = inner_type.to_rust_type(trait_checker);
                            trait_checker
                                .type_implements_trait(&rust_type, required_trait)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    // For naturally owned types (Int, Bool, etc.), check the type directly
                    _ => {
                        let rust_type = receiver_type.to_rust_type(trait_checker);
                        trait_checker
                            .type_implements_trait(&rust_type, required_trait)
                            .unwrap_or(false)
                    }
                }
            }
            SelfKind::None => {
                // Associated function - no receiver check needed
                true
            }
        }
    }

    /// Check if a method variant matches the receiver type
    fn method_matches_receiver(
        &self,
        method_kind: &BuiltinMethodKind,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        match method_kind {
            BuiltinMethodKind::TraitMethod {
                method_name,
                required_trait,
            } => {
                // Get the dynamic method signature to determine if receiver can provide access
                if let Some(rust_self_kind) =
                    self.get_dynamic_method_self_kind(method_name, receiver_type, trait_checker)
                {
                    // Check if the Veltrano receiver type can provide the required Rust access
                    // and if the underlying type implements the trait
                    self.receiver_can_provide_rust_access(
                        receiver_type,
                        &rust_self_kind,
                        required_trait,
                        trait_checker,
                    )
                } else {
                    // If dynamic lookup fails, we can't determine receiver requirements
                    // Default to false (method not available)
                    false
                }
            }
            BuiltinMethodKind::SpecialMethod {
                receiver_type_filter,
                ..
            } => self.type_filter_matches(receiver_type_filter, receiver_type),
        }
    }

    /// Check if a type filter matches the receiver type
    fn type_filter_matches(&self, filter: &TypeFilter, receiver_type: &VeltranoType) -> bool {
        match filter {
            TypeFilter::All => true,
            TypeFilter::TypeConstructors(constructors) => {
                constructors.contains(&receiver_type.constructor)
            }
        }
    }

    /// Compute the return type for a method
    fn compute_return_type(
        &self,
        method_kind: &BuiltinMethodKind,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> VeltranoType {
        let strategy = match method_kind {
            BuiltinMethodKind::TraitMethod {
                method_name,
                required_trait: _,
            } => {
                // For trait methods, try to get return type from dynamic lookup
                if let Some(dynamic_return_type) =
                    self.get_dynamic_method_return_type(method_name, receiver_type, trait_checker)
                {
                    return dynamic_return_type;
                }

                // If dynamic lookup fails, we can't determine the return type
                // This shouldn't happen if the method was found via method_matches_receiver
                &MethodReturnTypeStrategy::SameAsReceiver
            }
            BuiltinMethodKind::SpecialMethod {
                return_type_strategy,
                ..
            } => return_type_strategy,
        };

        match strategy {
            MethodReturnTypeStrategy::SameAsReceiver => receiver_type.clone(),
            MethodReturnTypeStrategy::RefToReceiver => VeltranoType::ref_(receiver_type.clone()),
            MethodReturnTypeStrategy::MutRefToReceiver => {
                VeltranoType::mut_ref(receiver_type.clone())
            }
            MethodReturnTypeStrategy::FixedType(fixed_type) => fixed_type.clone(),
            MethodReturnTypeStrategy::RefSemantics => {
                // Implement correct ref() semantics:
                // Own<T> → T, T → Ref<T>, MutRef<T> → Ref<MutRef<T>>
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
        }
    }

    /// Get return type for an imported method
    fn get_imported_method_return_type(
        &self,
        method_name: &str,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Option<VeltranoType> {
        // Use same lookup logic as is_imported_method_available
        // Get the appropriate type for method lookup
        let rust_type = receiver_type.to_rust_type(trait_checker);

        if let Ok(Some(method_info)) = trait_checker.query_method_signature(&rust_type, method_name)
        {
            // Check if the receiver can provide the required access
            if self.receiver_can_provide_rust_access_for_imported(
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
                            if let Ok(veltrano_return_type) =
                                method_info.return_type.to_veltrano_type()
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

    /// Check if a Veltrano receiver type can provide the required Rust access for imported methods
    /// This is similar to receiver_can_provide_rust_access but doesn't require trait checking
    /// since we already know the method exists from the imported signature
    pub fn receiver_can_provide_rust_access_for_imported(
        &self,
        receiver_type: &VeltranoType,
        rust_self_kind: &SelfKind,
        _trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        match rust_self_kind {
            SelfKind::Ref => {
                // Rust method takes &self - ONLY Ref<T> and naturally referenced types can provide this
                match &receiver_type.constructor {
                    TypeConstructor::Ref => true,
                    TypeConstructor::Own => false, // No auto-borrow from Own<T>
                    _ => true, // Naturally referenced types (String, etc.) can provide &self
                }
            }
            SelfKind::MutRef => {
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

    /// Get dynamic method self kind from Rust interop system
    fn get_dynamic_method_self_kind(
        &self,
        method_name: &str,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Option<SelfKind> {
        let rust_type = receiver_type.to_rust_type(trait_checker);

        if let Ok(Some(method_info)) = trait_checker.query_method_signature(&rust_type, method_name)
        {
            Some(method_info.self_kind)
        } else {
            None
        }
    }

    /// Get dynamic method return type from Rust interop system
    fn get_dynamic_method_return_type(
        &self,
        method_name: &str,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Option<VeltranoType> {
        let rust_type = receiver_type.to_rust_type(trait_checker);

        if let Ok(Some(method_info)) = trait_checker.query_method_signature(&rust_type, method_name)
        {
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
                        return method_info.return_type.to_veltrano_type().ok();
                    }
                }
            }

            // For non-clone methods, use normal conversion
            method_info.return_type.to_veltrano_type().ok()
        } else {
            None
        }
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
