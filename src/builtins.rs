/// Centralized built-in functions and methods for Veltrano
/// This module consolidates all built-in function definitions that were previously
/// scattered across codegen.rs, rust_interop.rs, and implicit in type checking
use crate::rust_interop::RustInteropRegistry;
use crate::type_checker::{FunctionSignature, MethodSignature, TypeConstructor, VeltranoType};
use std::collections::HashMap;

/// Categories of built-in functions
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunctionKind {
    /// Rust macros that skip type checking (variadic arguments)
    RustMacro { macro_name: String },
    /// Special functions with custom type checking rules
    SpecialFunction {
        function_name: String,
        parameters: Vec<VeltranoType>,
        return_type: VeltranoType,
    },
}

/// Categories of built-in methods
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinMethodKind {
    /// Universal methods that require trait checking
    TraitMethod {
        method_name: String,
        required_trait: String,
        parameters: Vec<VeltranoType>,
        return_type_strategy: MethodReturnTypeStrategy,
    },
    /// Special methods with custom logic
    SpecialMethod {
        method_name: String,
        receiver_type_filter: TypeFilter,
        parameters: Vec<VeltranoType>,
        return_type_strategy: MethodReturnTypeStrategy,
    },
}

/// Strategy for determining method return types
#[derive(Debug, Clone, PartialEq)]
pub enum MethodReturnTypeStrategy {
    /// Return the receiver type unchanged
    SameAsReceiver,
    /// Return a reference to the receiver type
    RefToReceiver,
    /// Return a mutable reference to the receiver type
    MutRefToReceiver,
    /// Return an owned version (Own<T> for naturally referenced types, T for value types)
    OwnedVersion,
    /// Return a specific type regardless of receiver
    FixedType(VeltranoType),
    /// For clone: return owned version based on trait checking
    CloneSemantics,
}

/// Filter for determining if a method applies to a receiver type
#[derive(Debug, Clone, PartialEq)]
pub enum TypeFilter {
    /// Method applies to all types
    All,
    /// Method applies only to specific type constructors
    TypeConstructors(Vec<TypeConstructor>),
    /// Method applies only to types that implement a specific trait
    HasTrait(String),
}

/// Registry for all built-in functions and methods
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunctionKind>,
    methods: HashMap<String, Vec<BuiltinMethodKind>>, // Method name -> list of variants
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
            methods: HashMap::new(),
        };

        registry.register_builtin_functions();
        registry.register_builtin_methods();

        registry
    }

    /// Register built-in functions
    fn register_builtin_functions(&mut self) {
        // Rust macros (variadic, skip type checking)
        let rust_macros = vec!["println", "print", "panic", "assert", "debug_assert"];
        for macro_name in rust_macros {
            self.functions.insert(
                macro_name.to_string(),
                BuiltinFunctionKind::RustMacro {
                    macro_name: macro_name.to_string(),
                },
            );
        }

        // Special functions with specific signatures
        self.functions.insert(
            "MutRef".to_string(),
            BuiltinFunctionKind::SpecialFunction {
                function_name: "MutRef".to_string(),
                parameters: vec![VeltranoType::custom("T".to_string())], // Generic parameter
                return_type: VeltranoType::mut_ref(VeltranoType::custom("T".to_string())),
            },
        );
    }

    /// Register built-in methods
    fn register_builtin_methods(&mut self) {
        // Universal trait methods
        self.register_method(
            "clone",
            BuiltinMethodKind::TraitMethod {
                method_name: "clone".to_string(),
                required_trait: "Clone".to_string(),
                parameters: vec![],
                return_type_strategy: MethodReturnTypeStrategy::CloneSemantics,
            },
        );

        self.register_method(
            "toString",
            BuiltinMethodKind::TraitMethod {
                method_name: "toString".to_string(),
                required_trait: "ToString".to_string(),
                parameters: vec![],
                return_type_strategy: MethodReturnTypeStrategy::FixedType(VeltranoType::own(
                    VeltranoType::string(),
                )),
            },
        );

        // Reference creation methods (available on all appropriate types)
        self.register_method(
            "ref",
            BuiltinMethodKind::SpecialMethod {
                method_name: "ref".to_string(),
                receiver_type_filter: TypeFilter::All,
                parameters: vec![],
                return_type_strategy: MethodReturnTypeStrategy::RefToReceiver,
            },
        );

        self.register_method(
            "mutRef",
            BuiltinMethodKind::SpecialMethod {
                method_name: "mutRef".to_string(),
                receiver_type_filter: TypeFilter::TypeConstructors(vec![
                    TypeConstructor::Own,
                    TypeConstructor::MutRef,
                ]),
                parameters: vec![],
                return_type_strategy: MethodReturnTypeStrategy::MutRefToReceiver,
            },
        );

        // Special methods
        self.register_method(
            "toSlice",
            BuiltinMethodKind::SpecialMethod {
                method_name: "toSlice".to_string(),
                receiver_type_filter: TypeFilter::TypeConstructors(vec![TypeConstructor::Vec]),
                parameters: vec![],
                return_type_strategy: MethodReturnTypeStrategy::RefToReceiver,
            },
        );

        // Bump allocation methods (available on all types)
        self.register_method(
            "bumpRef",
            BuiltinMethodKind::SpecialMethod {
                method_name: "bumpRef".to_string(),
                receiver_type_filter: TypeFilter::All,
                parameters: vec![],
                return_type_strategy: MethodReturnTypeStrategy::RefToReceiver,
            },
        );
    }

    /// Helper to register a method
    fn register_method(&mut self, method_name: &str, method_kind: BuiltinMethodKind) {
        self.methods
            .entry(method_name.to_string())
            .or_insert_with(Vec::new)
            .push(method_kind);
    }

    /// Check if a function is a built-in
    pub fn is_builtin_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get built-in function kind
    pub fn get_builtin_function(&self, name: &str) -> Option<&BuiltinFunctionKind> {
        self.functions.get(name)
    }

    /// Check if a method is a built-in
    pub fn is_builtin_method(&self, name: &str) -> bool {
        self.methods.contains_key(name)
    }

    /// Get built-in method variants
    pub fn get_builtin_methods(&self, name: &str) -> Option<&Vec<BuiltinMethodKind>> {
        self.methods.get(name)
    }

    /// Check if a function is a Rust macro (skips type checking)
    pub fn is_rust_macro(&self, name: &str) -> bool {
        if let Some(BuiltinFunctionKind::RustMacro { .. }) = self.functions.get(name) {
            true
        } else {
            false
        }
    }

    /// Get function signatures for type checker initialization
    pub fn get_function_signatures(&self) -> Vec<FunctionSignature> {
        let mut signatures = Vec::new();

        for (_name, kind) in &self.functions {
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

    /// Get method signatures for a specific receiver type (with trait checking)
    pub fn get_method_signatures_for_type(
        &self,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Vec<MethodSignature> {
        let mut signatures = Vec::new();

        for (_method_name, method_variants) in &self.methods {
            for method_kind in method_variants {
                if self.method_matches_receiver(method_kind, receiver_type, trait_checker) {
                    let method_name = match method_kind {
                        BuiltinMethodKind::TraitMethod { method_name, .. } => method_name,
                        BuiltinMethodKind::SpecialMethod { method_name, .. } => method_name,
                    };

                    let parameters = match method_kind {
                        BuiltinMethodKind::TraitMethod { parameters, .. } => parameters,
                        BuiltinMethodKind::SpecialMethod { parameters, .. } => parameters,
                    };

                    let return_type =
                        self.compute_return_type(method_kind, receiver_type, trait_checker);

                    signatures.push(MethodSignature {
                        name: method_name.clone(),
                        receiver_type: receiver_type.clone(),
                        parameters: parameters.clone(),
                        return_type,
                    });
                }
            }
        }

        signatures
    }

    /// Check if a method is available on a given receiver type (with trait checking)
    pub fn is_method_available(
        &self,
        method_name: &str,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        if let Some(method_variants) = self.methods.get(method_name) {
            for method_kind in method_variants {
                if self.method_matches_receiver(method_kind, receiver_type, trait_checker) {
                    return true;
                }
            }
        }
        false
    }

    /// Get the return type for a method call (with trait checking)
    pub fn get_method_return_type(
        &self,
        method_name: &str,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Option<VeltranoType> {
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
        None
    }

    /// Check if a method variant matches the receiver type
    fn method_matches_receiver(
        &self,
        method_kind: &BuiltinMethodKind,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        match method_kind {
            BuiltinMethodKind::TraitMethod { required_trait, .. } => {
                // Check if receiver type implements the required trait
                let rust_type_name = receiver_type.to_rust_type_name();
                trait_checker
                    .type_implements_trait(&rust_type_name, required_trait)
                    .unwrap_or(false)
            }
            BuiltinMethodKind::SpecialMethod {
                receiver_type_filter,
                ..
            } => self.type_filter_matches(receiver_type_filter, receiver_type, trait_checker),
        }
    }

    /// Check if a type filter matches the receiver type
    fn type_filter_matches(
        &self,
        filter: &TypeFilter,
        receiver_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> bool {
        match filter {
            TypeFilter::All => true,
            TypeFilter::TypeConstructors(constructors) => {
                constructors.contains(&receiver_type.constructor)
            }
            TypeFilter::HasTrait(trait_name) => {
                let rust_type_name = receiver_type.to_rust_type_name();
                trait_checker
                    .type_implements_trait(&rust_type_name, trait_name)
                    .unwrap_or(false)
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
                return_type_strategy,
                ..
            } => return_type_strategy,
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
            MethodReturnTypeStrategy::OwnedVersion => {
                if receiver_type.is_naturally_referenced(trait_checker) {
                    VeltranoType::own(receiver_type.clone())
                } else {
                    receiver_type.clone() // Already owned for value types
                }
            }
            MethodReturnTypeStrategy::FixedType(fixed_type) => fixed_type.clone(),
            MethodReturnTypeStrategy::CloneSemantics => {
                // Clone returns an owned version based on trait checking
                if receiver_type.is_naturally_referenced(trait_checker) {
                    VeltranoType::own(receiver_type.clone())
                } else {
                    receiver_type.clone() // Already owned for value types
                }
            }
        }
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
