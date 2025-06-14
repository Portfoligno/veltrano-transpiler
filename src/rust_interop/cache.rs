//! Caching logic for Rust type information

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Information about a Rust crate extracted from rustdoc JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub functions: HashMap<String, FunctionInfo>,
    pub types: HashMap<String, TypeInfo>,
    pub traits: HashMap<String, TraitInfo>,
    /// Maps type name -> set of implemented trait names
    pub trait_implementations: HashMap<String, HashSet<String>>,
}

/// Information about a Rust function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub full_path: String,
    pub generics: Vec<GenericParam>,
    pub parameters: Vec<Parameter>,
    pub return_type: RustTypeSignature,
    pub is_unsafe: bool,
    pub is_const: bool,
    pub documentation: Option<String>,
}

/// Information about a Rust type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub full_path: String,
    pub kind: TypeKind,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<MethodInfo>,
    pub fields: Vec<FieldInfo>,     // For structs
    pub variants: Vec<VariantInfo>, // For enums
}

/// Information about a Rust trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitInfo {
    pub name: String,
    pub full_path: String,
    pub methods: Vec<MethodInfo>,
    pub associated_types: Vec<String>,
}

/// Kind of Rust type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    Struct,
    Enum,
    Union,
    Trait,
    TypeAlias,
}

/// Information about a method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodInfo {
    pub name: String,
    pub self_kind: super::types::SelfKind,
    pub generics: Vec<GenericParam>,
    pub parameters: Vec<Parameter>,
    pub return_type: RustTypeSignature,
    pub is_unsafe: bool,
    pub is_const: bool,
}

/// Information about a struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: RustTypeSignature,
    pub is_public: bool,
}

/// Information about an enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

/// Information about a function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: RustTypeSignature,
}

/// Information about a generic parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,
    pub default: Option<String>,
}

/// A parsed Rust type signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustTypeSignature {
    pub raw: String,                            // "Option<&'a str>"
    pub parsed: Option<super::types::RustType>, // Our parsed representation
    pub lifetimes: Vec<String>,                 // ["'a"]
    pub bounds: Vec<String>,                    // Trait bounds like "T: Clone"
}
