//! Type definitions for builtin functions and methods
//!
//! This module contains all the enum types used to categorize and configure
//! builtin functions and methods in Veltrano.

use crate::types::{TypeConstructor, VeltranoType};

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
    /// Return a specific type regardless of receiver
    FixedType(VeltranoType),
    /// For ref(): Own<T> → T, T → Ref<T>
    RefSemantics,
}

/// Filter for determining if a method applies to a receiver type
#[derive(Debug, Clone, PartialEq)]
pub enum TypeFilter {
    /// Method applies to all types
    All,
    /// Method applies only to specific type constructors
    TypeConstructors(Vec<TypeConstructor>),
}
