//! Type definitions for builtin functions and methods
//!
//! This module contains all the enum types used to categorize and configure
//! builtin functions and methods in Veltrano.

use crate::types::VeltranoType;

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

/// Special operator methods that generate operators instead of function calls
#[derive(Debug, Clone, PartialEq)]
pub enum OperatorMethod {
    /// ref() method - generates &
    Ref,
    /// mutRef() method - generates &mut
    MutRef,
    /// bumpRef() method - generates bump.alloc()
    BumpRef,
}

impl OperatorMethod {
    /// Get the method name as it appears in Veltrano code
    pub fn method_name(&self) -> &'static str {
        match self {
            OperatorMethod::Ref => "ref",
            OperatorMethod::MutRef => "mutRef",
            OperatorMethod::BumpRef => "bumpRef",
        }
    }
}

/// Categories of built-in methods
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinMethodKind {
    /// Operator methods that generate operators instead of function calls
    Operator(OperatorMethod),
}
