/// Centralized built-in functions and methods for Veltrano
/// This module consolidates all built-in function definitions that were previously
/// scattered across codegen.rs, rust_interop.rs, and implicit in type checking
use crate::type_checker::{
    FunctionSignature, MethodSignature, Mutability, Ownership, VeltranoBaseType, VeltranoType,
};
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
        return_type_fn: fn(&VeltranoType) -> VeltranoType, // Function to compute return type based on receiver
    },
    /// Ownership conversion methods (always available on owned types)
    OwnershipConversion {
        method_name: String,
        source_ownership: Ownership,
        target_ownership: Ownership,
        return_type_fn: fn(&VeltranoType) -> VeltranoType,
    },
    /// Special methods with custom logic
    SpecialMethod {
        method_name: String,
        receiver_type_filter: fn(&VeltranoType) -> bool, // Whether this method applies to the receiver type
        parameters: Vec<VeltranoType>,
        return_type_fn: fn(&VeltranoType) -> VeltranoType,
    },
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
                parameters: vec![VeltranoType {
                    base: VeltranoBaseType::Custom("T".to_string()), // Generic parameter
                    ownership: Ownership::Owned,
                    mutability: Mutability::Immutable,
                }],
                return_type: VeltranoType {
                    base: VeltranoBaseType::Custom("T".to_string()),
                    ownership: Ownership::MutBorrowed,
                    mutability: Mutability::Mutable,
                },
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
                return_type_fn: |receiver| {
                    // clone() returns an owned version of the type
                    VeltranoType {
                        base: receiver.base.clone(),
                        ownership: Ownership::Owned,
                        mutability: Mutability::Immutable,
                    }
                },
            },
        );

        self.register_method(
            "toString",
            BuiltinMethodKind::TraitMethod {
                method_name: "toString".to_string(),
                required_trait: "ToString".to_string(),
                parameters: vec![],
                return_type_fn: |_| VeltranoType {
                    base: VeltranoBaseType::String,
                    ownership: Ownership::Owned,
                    mutability: Mutability::Immutable,
                },
            },
        );

        // Reference creation methods (available on all appropriate types)
        self.register_method(
            "ref",
            BuiltinMethodKind::SpecialMethod {
                method_name: "ref".to_string(),
                receiver_type_filter: |_| true, // Available on all types
                parameters: vec![],
                return_type_fn: |receiver| {
                    match &receiver.ownership {
                        Ownership::Owned => {
                            // Own<T> → T (borrowing the owned value)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::Borrowed,
                                mutability: Mutability::Immutable,
                            }
                        }
                        Ownership::Borrowed => {
                            // T → Ref<T> (taking reference of borrowed value)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::Borrowed,
                                mutability: Mutability::Immutable,
                            }
                        }
                        Ownership::MutBorrowed => {
                            // MutRef<T> → Ref<MutRef<T>> (immutable reference to mutable reference)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::Borrowed,
                                mutability: Mutability::Immutable,
                            }
                        }
                    }
                },
            },
        );

        self.register_method(
            "mutRef",
            BuiltinMethodKind::SpecialMethod {
                method_name: "mutRef".to_string(),
                receiver_type_filter: |receiver| {
                    // Available on owned types and mutable references
                    matches!(
                        receiver.ownership,
                        Ownership::Owned | Ownership::MutBorrowed
                    )
                },
                parameters: vec![],
                return_type_fn: |receiver| {
                    match &receiver.ownership {
                        Ownership::Owned => {
                            // Own<T> → MutRef<Own<T>> (mutable reference to owned value)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::MutBorrowed,
                                mutability: Mutability::Mutable,
                            }
                        }
                        Ownership::MutBorrowed => {
                            // MutRef<T> → MutRef<MutRef<T>> (mutable reference to mutable reference)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::MutBorrowed,
                                mutability: Mutability::Mutable,
                            }
                        }
                        Ownership::Borrowed => {
                            // This case is filtered out, but included for completeness
                            receiver.clone()
                        }
                    }
                },
            },
        );

        // Special methods
        self.register_method(
            "toSlice",
            BuiltinMethodKind::SpecialMethod {
                method_name: "toSlice".to_string(),
                receiver_type_filter: |receiver| {
                    // toSlice is available on Vec types
                    matches!(receiver.base, VeltranoBaseType::Vec(_))
                },
                parameters: vec![],
                return_type_fn: |receiver| {
                    if let VeltranoBaseType::Vec(element_type) = &receiver.base {
                        VeltranoType {
                            base: VeltranoBaseType::Slice(element_type.clone()),
                            ownership: Ownership::Borrowed,
                            mutability: Mutability::Immutable,
                        }
                    } else {
                        // Fallback (should not happen due to filter)
                        receiver.clone()
                    }
                },
            },
        );

        // Bump allocation methods (available on all types)
        self.register_method(
            "bumpRef",
            BuiltinMethodKind::SpecialMethod {
                method_name: "bumpRef".to_string(),
                receiver_type_filter: |_| true, // Available on all types for bump allocation
                parameters: vec![],
                return_type_fn: |receiver| {
                    // bumpRef creates a bump-allocated reference with same ownership pattern as ref()
                    match &receiver.ownership {
                        Ownership::Owned => {
                            // Own<T> → T (bump-allocated borrowing)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::Borrowed,
                                mutability: Mutability::Immutable,
                            }
                        }
                        Ownership::Borrowed => {
                            // T → Ref<T> (bump-allocated reference)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::Borrowed,
                                mutability: Mutability::Immutable,
                            }
                        }
                        Ownership::MutBorrowed => {
                            // MutRef<T> → Ref<MutRef<T>> (bump-allocated immutable reference)
                            VeltranoType {
                                base: receiver.base.clone(),
                                ownership: Ownership::Borrowed,
                                mutability: Mutability::Immutable,
                            }
                        }
                    }
                },
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

        for (name, kind) in &self.functions {
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

    /// Get method signatures for a specific receiver type
    pub fn get_method_signatures_for_type(
        &self,
        receiver_type: &VeltranoType,
    ) -> Vec<MethodSignature> {
        let mut signatures = Vec::new();

        for (method_name, method_variants) in &self.methods {
            for method_kind in method_variants {
                match method_kind {
                    BuiltinMethodKind::TraitMethod {
                        method_name,
                        parameters,
                        return_type_fn,
                        ..
                    } => {
                        // Note: trait checking will be done separately
                        signatures.push(MethodSignature {
                            name: method_name.clone(),
                            receiver_type: receiver_type.clone(),
                            parameters: parameters.clone(),
                            return_type: return_type_fn(receiver_type),
                        });
                    }
                    BuiltinMethodKind::OwnershipConversion {
                        method_name,
                        source_ownership,
                        return_type_fn,
                        ..
                    } => {
                        // Only include if the receiver has the required ownership
                        if receiver_type.ownership == *source_ownership {
                            signatures.push(MethodSignature {
                                name: method_name.clone(),
                                receiver_type: receiver_type.clone(),
                                parameters: vec![],
                                return_type: return_type_fn(receiver_type),
                            });
                        }
                    }
                    BuiltinMethodKind::SpecialMethod {
                        method_name,
                        receiver_type_filter,
                        parameters,
                        return_type_fn,
                    } => {
                        // Only include if the receiver type passes the filter
                        if receiver_type_filter(receiver_type) {
                            signatures.push(MethodSignature {
                                name: method_name.clone(),
                                receiver_type: receiver_type.clone(),
                                parameters: parameters.clone(),
                                return_type: return_type_fn(receiver_type),
                            });
                        }
                    }
                }
            }
        }

        signatures
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
