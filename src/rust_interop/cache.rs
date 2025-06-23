//! Caching logic for Rust type information

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Distinguishes between different kinds of items for case conversion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Function, // Covers both free functions and methods
    Constant, // Covers both module constants and associated constants
    Static,   // Static items
}

/// A crate name (e.g., "std", "my_crate")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrateName(pub String);

impl CrateName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CrateName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for CrateName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Base location for any Rust item
/// (crate_name, module_path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustModulePath(pub CrateName, pub Vec<String>);

impl RustModulePath {
    pub fn crate_name(&self) -> &CrateName {
        &self.0
    }

    pub fn module_path(&self) -> &[String] {
        &self.1
    }
}

/// A Rust type (can be nested)
/// (module_path, type_path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustTypePath(pub RustModulePath, pub Vec<String>);

impl RustTypePath {
    /// Add a nested type component (e.g., HashMap -> HashMap::Entry)
    pub fn with_nested(mut self, name: String) -> Self {
        self.1.push(name);
        self
    }

    /// Convert to full Rust path syntax
    pub fn to_rust_syntax(&self) -> String {
        let mut parts = vec![self.0.crate_name().as_str().to_string()];
        parts.extend(self.0.module_path().iter().cloned());
        parts.extend(self.1.clone());
        parts.join("::")
    }

    pub fn module_path(&self) -> &RustModulePath {
        &self.0
    }

    pub fn type_path(&self) -> &[String] {
        &self.1
    }
}

/// A fully resolved path to a Rust item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustPath {
    /// Module-level function or constant (e.g., std::process::exit)
    /// (module_path, name, kind)
    ModuleItem(RustModulePath, String, ItemKind),

    /// A type (e.g., std::collections::HashMap)
    Type(RustTypePath),

    /// An enum variant (e.g., Option::Some, Result::Ok)
    /// (enum_type, variant_name)
    EnumVariant(RustTypePath, String),

    /// A method or associated item (e.g., Vec::new, HashMap::Entry::or_insert)
    /// (parent_type, item_name, kind)
    TypeItem(RustTypePath, String, ItemKind),
}

impl RustPath {
    /// Convert to Rust syntax when needed (e.g., for code generation)
    pub fn to_rust_syntax(&self) -> String {
        match self {
            RustPath::ModuleItem(module_path, name, _) => {
                let mut parts = vec![module_path.crate_name().as_str().to_string()];
                parts.extend(module_path.module_path().iter().cloned());
                parts.push(name.clone());
                parts.join("::")
            }
            RustPath::Type(type_path) => type_path.to_rust_syntax(),
            RustPath::EnumVariant(enum_type, variant_name) => {
                format!("{}::{}", enum_type.to_rust_syntax(), variant_name)
            }
            RustPath::TypeItem(parent_type, item_name, _) => {
                format!("{}::{}", parent_type.to_rust_syntax(), item_name)
            }
        }
    }

    /// Get the module path (crate + modules)
    pub fn module_path(&self) -> &RustModulePath {
        match self {
            RustPath::ModuleItem(module_path, _, _) => module_path,
            RustPath::Type(type_path) => type_path.module_path(),
            RustPath::EnumVariant(enum_type, _) => enum_type.module_path(),
            RustPath::TypeItem(parent_type, _, _) => parent_type.module_path(),
        }
    }

    /// Check if case conversion should be applied (snake_case to camelCase)
    pub fn should_convert_case(&self) -> bool {
        match self {
            RustPath::ModuleItem(_, _, kind) => matches!(kind, ItemKind::Function),
            RustPath::TypeItem(_, _, kind) => matches!(kind, ItemKind::Function),
            RustPath::Type(_) | RustPath::EnumVariant(_, _) => false,
        }
    }
}

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
    pub path: RustPath,
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
    pub path: RustPath,
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
    pub path: RustPath,
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
