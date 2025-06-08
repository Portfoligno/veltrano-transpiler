/// Module for handling Rust interoperability and type signature extraction
/// This provides mechanisms to:
/// 1. Declare external Rust functions/methods
/// 2. Parse Rust type signatures
/// 3. Convert between Rust and Veltrano type representations
/// 4. Dynamically query Rust toolchain for type information
use crate::types::VeltranoType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents an external Rust item (function, method, or type)
#[derive(Debug, Clone)]
pub enum ExternItem {
    Function {
        name: String,
        path: String, // Full Rust path e.g., "std::vec::Vec::new"
        params: Vec<(String, RustType)>,
        return_type: RustType,
        is_unsafe: bool,
    },
    Method {
        type_name: String,
        method_name: String,
        self_kind: SelfKind,
        params: Vec<(String, RustType)>,
        return_type: RustType,
        is_unsafe: bool,
    },
    Type {
        name: String,
        rust_path: String,
        generic_params: Vec<String>,
    },
}

/// How a method takes self
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelfKind {
    None,   // Associated function (no self)
    Value,  // self
    Ref,    // &self
    MutRef, // &mut self
}

/// Rust type representation that can be converted to Veltrano types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustType {
    // Primitive types
    I32,
    I64,
    ISize,
    U32,
    U64,
    USize,
    Bool,
    Char,
    Unit,
    Never,

    // String types
    Str,
    String,

    // Reference types
    Ref {
        lifetime: Option<String>,
        inner: Box<RustType>,
    },
    MutRef {
        lifetime: Option<String>,
        inner: Box<RustType>,
    },

    // Smart pointers
    Box(Box<RustType>),
    Rc(Box<RustType>),
    Arc(Box<RustType>),

    // Generic types
    Vec(Box<RustType>),
    Option(Box<RustType>),
    Result {
        ok: Box<RustType>,
        err: Box<RustType>,
    },

    // Custom types
    Custom {
        name: String,
        generics: Vec<RustType>,
    },

    // Generic parameter
    Generic(String),
}

/// Information about an imported method with full signature details
#[derive(Debug, Clone)]
pub struct ImportedMethodInfo {
    pub method_name: String,
    pub self_kind: SelfKind,
    pub parameters: Vec<RustType>,
    pub return_type: RustType, // The actual parsed return type from Rust
    pub trait_name: Option<String>, // Which trait this method comes from (if any)
}

impl RustType {
    /// Convert a Rust type to a Veltrano type
    pub fn to_veltrano_type(&self) -> Result<VeltranoType, String> {
        match self {
            // Primitives
            RustType::I32 => Ok(VeltranoType::i32()),
            RustType::I64 => Ok(VeltranoType::i64()),
            RustType::ISize => Ok(VeltranoType::isize()),
            RustType::U32 => Ok(VeltranoType::u32()),
            RustType::U64 => Ok(VeltranoType::u64()),
            RustType::USize => Ok(VeltranoType::usize()),
            RustType::Char => Ok(VeltranoType::char()),
            RustType::Bool => Ok(VeltranoType::bool()),
            RustType::Unit => Ok(VeltranoType::unit()),
            RustType::Never => Ok(VeltranoType::nothing()),

            // String types
            RustType::Str => Ok(VeltranoType::str()),
            RustType::String => Ok(VeltranoType::string()),

            // References
            RustType::Ref { inner, .. } => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::ref_(inner_type))
            }
            RustType::MutRef { inner, .. } => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::mut_ref(inner_type))
            }

            // Smart pointers
            RustType::Box(inner) => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::boxed(inner_type))
            }

            // Generic types
            RustType::Vec(inner) => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::vec(inner_type))
            }
            RustType::Option(inner) => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::option(inner_type))
            }
            RustType::Result { ok, err } => {
                let ok_type = ok.to_veltrano_type()?;
                let err_type = err.to_veltrano_type()?;
                Ok(VeltranoType::result(ok_type, err_type))
            }

            // Custom types
            RustType::Custom { name, .. } => Ok(VeltranoType::custom(name.clone())),

            // Generic parameters
            RustType::Generic(name) => Ok(VeltranoType::custom(format!("${}", name))), // Prefix with $ to indicate generic

            _ => Err(format!("Unsupported Rust type for conversion: {:?}", self)),
        }
    }
}

/// Registry for external Rust items
#[derive(Debug)]
pub struct RustInteropRegistry {
    items: HashMap<String, ExternItem>,
    dynamic_registry: DynamicRustRegistry,
}

impl RustInteropRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            items: HashMap::new(),
            dynamic_registry: DynamicRustRegistry::new(),
        };
        registry.register_stdlib();
        registry
    }

    /// Register standard library items that Veltrano code commonly uses
    fn register_stdlib(&mut self) {
        // println! macro
        self.register(ExternItem::Function {
            name: "println".to_string(),
            path: "std::println!".to_string(),
            params: vec![(
                "format".to_string(),
                RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                },
            )],
            return_type: RustType::Unit,
            is_unsafe: false,
        });

        // Vec::new
        self.register(ExternItem::Method {
            type_name: "Vec".to_string(),
            method_name: "new".to_string(),
            self_kind: SelfKind::None,
            params: vec![],
            return_type: RustType::Vec(Box::new(RustType::Generic("T".to_string()))),
            is_unsafe: false,
        });

        // String::from
        self.register(ExternItem::Method {
            type_name: "String".to_string(),
            method_name: "from".to_string(),
            self_kind: SelfKind::None,
            params: vec![(
                "s".to_string(),
                RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                },
            )],
            return_type: RustType::String,
            is_unsafe: false,
        });
    }

    pub fn register(&mut self, item: ExternItem) {
        let key = match &item {
            ExternItem::Function { name, .. } => name.clone(),
            ExternItem::Method {
                type_name,
                method_name,
                ..
            } => {
                format!("{}::{}", type_name, method_name)
            }
            ExternItem::Type { name, .. } => format!("type::{}", name),
        };
        self.items.insert(key, item);
    }

    pub fn get_function(&self, name: &str) -> Option<&ExternItem> {
        self.items.get(name)
    }

    pub fn get_method(&self, type_name: &str, method_name: &str) -> Option<&ExternItem> {
        let key = format!("{}::{}", type_name, method_name);
        self.items.get(&key)
    }

    /// Check if a type implements a specific trait
    pub fn type_implements_trait(
        &mut self,
        type_path: &str,
        trait_name: &str,
    ) -> Result<bool, RustInteropError> {
        // For built-in types, we can have hardcoded knowledge
        let implements = match type_path {
            // Primitive types that implement Copy and Clone
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "bool" | "char" => {
                matches!(
                    trait_name,
                    "Clone" | "Copy" | "Debug" | "Display" | "ToString"
                )
            }
            // String types
            "String" | "std::string::String" => {
                matches!(trait_name, "Clone" | "Debug" | "Display" | "ToString")
            }
            "&str" | "str" => {
                matches!(trait_name, "Debug" | "Display" | "ToString")
            }
            // Unit type
            "()" => {
                matches!(trait_name, "Clone" | "Copy" | "Debug")
            }
            // For other types, we don't have knowledge yet
            _ => false,
        };

        Ok(implements)
    }

    /// Get method information for a type by querying available methods
    /// This includes both inherent methods and trait methods
    pub fn get_method_info(
        &self,
        _type_path: &str,
        _method_name: &str,
    ) -> Option<ImportedMethodInfo> {
        // No hardcoded method signatures - rely entirely on dynamic registry
        None
    }

    /// Query method signature dynamically from crate metadata
    /// This integrates with the DynamicRustRegistry for full method resolution
    pub fn query_method_signature(
        &mut self,
        type_path: &str,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, RustInteropError> {
        // First try hardcoded method info for built-in types
        if let Some(method_info) = self.get_method_info(type_path, method_name) {
            return Ok(Some(method_info));
        }

        // For other types, use the dynamic registry to query method signatures
        self.query_dynamic_method_signature(type_path, method_name)
    }

    /// Query method signature using the dynamic registry system
    fn query_dynamic_method_signature(
        &mut self,
        type_path: &str,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, RustInteropError> {
        // Parse the type_path to determine which crate it comes from
        // For standard library types like "i64", "String", we assume they're from "std"
        let crate_name = if self.is_stdlib_type(type_path) {
            "std"
        } else if type_path.contains("::") {
            // For paths like "serde::Serialize", extract the crate name
            type_path.split("::").next().unwrap_or("unknown")
        } else {
            // For simple names, assume they're from the current crate or std
            "std"
        };

        // Try to get type information from the dynamic registry
        let full_type_path = if type_path.contains("::") {
            type_path.to_string()
        } else {
            format!("{}::{}", crate_name, type_path)
        };

        if let Ok(Some(type_info)) = self.dynamic_registry.get_type(&full_type_path) {
            // Look for the method in the type's inherent methods
            for method in &type_info.methods {
                if method.name == method_name {
                    return Ok(Some(ImportedMethodInfo {
                        method_name: method.name.clone(),
                        self_kind: method.self_kind.clone(),
                        parameters: self.convert_parameters(&method.parameters),
                        return_type: self.convert_rust_type_signature(&method.return_type),
                        trait_name: None, // Inherent method
                    }));
                }
            }
        }

        // If not found in inherent methods, this is where we'd search trait implementations
        // For now, return None for methods we can't find
        Ok(None)
    }

    /// Check if a type is a standard library type
    fn is_stdlib_type(&self, type_path: &str) -> bool {
        matches!(
            type_path,
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            | "f32" | "f64" | "bool" | "char" | "str" | "String"
            | "()" | "Vec" | "Option" | "Result"
        )
    }

    /// Convert method parameters from the dynamic registry format
    fn convert_parameters(&self, parameters: &[Parameter]) -> Vec<RustType> {
        parameters.iter()
            .filter_map(|param| param.param_type.parsed.clone())
            .collect()
    }

    /// Convert return type from the dynamic registry format
    fn convert_rust_type_signature(&self, return_type: &RustTypeSignature) -> RustType {
        return_type.parsed.clone().unwrap_or(RustType::Unit)
    }
}

/// Simple parser for Rust type signatures
/// This is a basic implementation - a full parser would need proper tokenization
pub struct RustTypeParser;

impl RustTypeParser {
    /// Parse a simple Rust type string
    pub fn parse(type_str: &str) -> Result<RustType, String> {
        let trimmed = type_str.trim();

        // Handle references
        if let Some(rest) = trimmed.strip_prefix("&mut ") {
            return Ok(RustType::MutRef {
                lifetime: None,
                inner: Box::new(Self::parse(rest)?),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("&") {
            // Check for lifetime
            let (lifetime, rest) = if rest.starts_with('\'') {
                let end = rest.find(' ').unwrap_or(rest.len());
                let lifetime = rest[1..end].to_string();
                let remaining = if end < rest.len() {
                    rest[end..].trim()
                } else {
                    ""
                };
                (Some(lifetime), remaining)
            } else {
                (None, rest)
            };

            return Ok(RustType::Ref {
                lifetime,
                inner: Box::new(Self::parse(rest)?),
            });
        }

        // Handle Box<T>
        if let Some(inner) = trimmed
            .strip_prefix("Box<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(RustType::Box(Box::new(Self::parse(inner)?)));
        }

        // Handle Vec<T>
        if let Some(inner) = trimmed
            .strip_prefix("Vec<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(RustType::Vec(Box::new(Self::parse(inner)?)));
        }

        // Handle Option<T>
        if let Some(inner) = trimmed
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(RustType::Option(Box::new(Self::parse(inner)?)));
        }

        // Handle basic types
        match trimmed {
            "i32" => Ok(RustType::I32),
            "i64" => Ok(RustType::I64),
            "isize" => Ok(RustType::ISize),
            "u32" => Ok(RustType::U32),
            "u64" => Ok(RustType::U64),
            "usize" => Ok(RustType::USize),
            "bool" => Ok(RustType::Bool),
            "char" => Ok(RustType::Char),
            "()" => Ok(RustType::Unit),
            "!" => Ok(RustType::Never),
            "str" => Ok(RustType::Str),
            "String" => Ok(RustType::String),
            _ => {
                // Assume it's a custom type or generic parameter
                if trimmed.len() == 1 && trimmed.chars().next().unwrap().is_uppercase() {
                    Ok(RustType::Generic(trimmed.to_string()))
                } else {
                    Ok(RustType::Custom {
                        name: trimmed.to_string(),
                        generics: vec![],
                    })
                }
            }
        }
    }
}

// === Dynamic Rust Interop System ===

/// Error types for dynamic Rust interop operations
#[derive(Debug, Clone)]
pub enum RustInteropError {
    CargoError(String),
    ParseError(String),
    IoError(String),
    SerdeError(String),
    CrateNotFound(String),
}

impl std::fmt::Display for RustInteropError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RustInteropError::CargoError(msg) => write!(f, "Cargo error: {}", msg),
            RustInteropError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            RustInteropError::IoError(msg) => write!(f, "IO error: {}", msg),
            RustInteropError::SerdeError(msg) => write!(f, "Serialization error: {}", msg),
            RustInteropError::CrateNotFound(name) => write!(f, "Crate not found: {}", name),
        }
    }
}

impl std::error::Error for RustInteropError {}

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
    pub self_kind: SelfKind,
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
    pub raw: String,              // "Option<&'a str>"
    pub parsed: Option<RustType>, // Our parsed representation
    pub lifetimes: Vec<String>,   // ["'a"]
    pub bounds: Vec<String>,      // Trait bounds like "T: Clone"
}

/// Trait for querying Rust type information
pub trait RustQuerier: std::fmt::Debug {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError>;
    fn supports_crate(&self, crate_name: &str) -> bool;
    fn priority(&self) -> u32; // Higher priority queriers tried first
}

/// rustdoc JSON-based querier - Phase 1 implementation
#[derive(Debug)]
pub struct RustdocQuerier {
    output_dir: PathBuf,
    cache: HashMap<String, CrateInfo>,
}

impl RustdocQuerier {
    pub fn new(output_dir: Option<PathBuf>) -> Self {
        let output_dir =
            output_dir.unwrap_or_else(|| std::env::temp_dir().join("veltrano_rustdoc"));

        Self {
            output_dir,
            cache: HashMap::new(),
        }
    }

    /// Extract crate information using rustdoc JSON output
    pub fn extract_crate_info(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        // Check cache first
        if let Some(cached) = self.cache.get(crate_name) {
            return Ok(cached.clone());
        }

        // Create output directory
        fs::create_dir_all(&self.output_dir).map_err(|e| {
            RustInteropError::IoError(format!("Failed to create output dir: {}", e))
        })?;

        // Run cargo doc with JSON output
        let output = Command::new("cargo")
            .args([
                "doc",
                "--output-format",
                "json",
                "--no-deps",
                "--target-dir",
                self.output_dir.to_str().unwrap(),
                "--package",
                crate_name,
            ])
            .output()
            .map_err(|e| RustInteropError::CargoError(format!("Failed to run cargo doc: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RustInteropError::CargoError(format!(
                "cargo doc failed: {}",
                stderr
            )));
        }

        // Parse the generated JSON
        let json_dir = self.output_dir.join("doc");
        let crate_info = self.parse_rustdoc_output(&json_dir, crate_name)?;

        // Cache the result
        self.cache
            .insert(crate_name.to_string(), crate_info.clone());

        Ok(crate_info)
    }

    fn parse_rustdoc_output(
        &self,
        _json_dir: &Path,
        crate_name: &str,
    ) -> Result<CrateInfo, RustInteropError> {
        // For now, return a placeholder - full rustdoc JSON parsing would be quite extensive
        // This is where we'd parse the actual rustdoc JSON format
        Ok(CrateInfo {
            name: crate_name.to_string(),
            version: "unknown".to_string(),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        })
    }
}

impl RustQuerier for RustdocQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        self.extract_crate_info(crate_name)
    }

    fn supports_crate(&self, _crate_name: &str) -> bool {
        true // rustdoc can handle any crate
    }

    fn priority(&self) -> u32 {
        100 // High priority - most reliable
    }
}

/// Unified interface for dynamic Rust type querying
#[derive(Debug)]
pub struct DynamicRustRegistry {
    pub queriers: Vec<Box<dyn RustQuerier>>,
    cache: HashMap<String, CrateInfo>,
}

impl DynamicRustRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            queriers: Vec::new(),
            cache: HashMap::new(),
        };

        // Add queriers in priority order (highest priority first)
        registry.add_querier(Box::new(RustdocQuerier::new(None)));

        // Add SynQuerier if possible (may fail if not in a Cargo project)
        if let Ok(syn_querier) = SynQuerier::new(None) {
            registry.add_querier(Box::new(syn_querier));
        }

        registry
    }

    pub fn add_querier(&mut self, querier: Box<dyn RustQuerier>) {
        // Insert in priority order (highest first)
        let priority = querier.priority();
        let insert_pos = self
            .queriers
            .iter()
            .position(|q| q.priority() < priority)
            .unwrap_or(self.queriers.len());

        self.queriers.insert(insert_pos, querier);
    }

    pub fn get_function(&mut self, path: &str) -> Result<Option<FunctionInfo>, RustInteropError> {
        let (crate_name, function_path) = self.parse_path(path)?;
        let crate_name = crate_name.to_string();
        let function_path = function_path.to_string();

        // Try cache first
        if let Some(cached) = self.cache.get(&crate_name) {
            return Ok(cached.functions.get(&function_path).cloned());
        }

        // Query dynamically
        for querier in &mut self.queriers {
            if querier.supports_crate(&crate_name) {
                match querier.query_crate(&crate_name) {
                    Ok(crate_info) => {
                        let result = crate_info.functions.get(&function_path).cloned();
                        // Cache the result
                        self.cache.insert(crate_name, crate_info);
                        return Ok(result);
                    }
                    Err(_) => continue, // Try next querier
                }
            }
        }

        Ok(None)
    }

    pub fn get_type(&mut self, path: &str) -> Result<Option<TypeInfo>, RustInteropError> {
        let (crate_name, type_path) = self.parse_path(path)?;
        let crate_name = crate_name.to_string();
        let type_path = type_path.to_string();

        // Try cache first
        if let Some(cached) = self.cache.get(&crate_name) {
            return Ok(cached.types.get(&type_path).cloned());
        }

        // Query dynamically
        for querier in &mut self.queriers {
            if querier.supports_crate(&crate_name) {
                match querier.query_crate(&crate_name) {
                    Ok(crate_info) => {
                        let result = crate_info.types.get(&type_path).cloned();
                        // Cache the result
                        self.cache.insert(crate_name, crate_info);
                        return Ok(result);
                    }
                    Err(_) => continue, // Try next querier
                }
            }
        }

        Ok(None)
    }

    /// Check if a type implements a specific trait
    pub fn type_implements_trait(
        &mut self,
        type_path: &str,
        trait_name: &str,
    ) -> Result<bool, RustInteropError> {
        // For built-in types, delegate to RustInteropRegistry
        let mut basic_registry = RustInteropRegistry::new();
        let basic_result = basic_registry.type_implements_trait(type_path, trait_name);
        if basic_result.is_ok() {
            return basic_result;
        }

        // For other types, query the crate info
        // Try to determine which crate the type belongs to
        let (crate_name, type_name) = if type_path.contains("::") {
            self.parse_path(type_path)?
        } else {
            // Assume it's in the current crate
            ("self", type_path)
        };

        // Query the crate
        let crate_name = crate_name.to_string();

        // Try cache first
        if let Some(cached) = self.cache.get(&crate_name) {
            return Ok(cached
                .trait_implementations
                .get(type_name)
                .map_or(false, |traits| traits.contains(trait_name)));
        }

        // Query dynamically
        for querier in &mut self.queriers {
            if querier.supports_crate(&crate_name) {
                match querier.query_crate(&crate_name) {
                    Ok(crate_info) => {
                        let result = crate_info
                            .trait_implementations
                            .get(type_name)
                            .map_or(false, |traits| traits.contains(trait_name));
                        // Cache the result
                        self.cache.insert(crate_name, crate_info);
                        return Ok(result);
                    }
                    Err(_) => continue, // Try next querier
                }
            }
        }

        Ok(false) // Default to not implemented if we can't find info
    }

    /// Get all traits implemented by a type
    pub fn get_implemented_traits(
        &mut self,
        type_path: &str,
    ) -> Result<Vec<String>, RustInteropError> {
        // For built-in types, return hardcoded list
        let traits = match type_path {
            // Primitive types
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "bool" | "char" => vec![
                "Clone",
                "Copy",
                "Debug",
                "Default",
                "PartialEq",
                "Eq",
                "PartialOrd",
                "Ord",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            // String types
            "String" | "std::string::String" => vec![
                "Clone",
                "Debug",
                "Display",
                "Default",
                "PartialEq",
                "Eq",
                "PartialOrd",
                "Ord",
                "ToString",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "&str" | "str" => vec![
                "Debug",
                "Display",
                "PartialEq",
                "Eq",
                "PartialOrd",
                "Ord",
                "ToString",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            // Unit type
            "()" => vec![
                "Clone",
                "Copy",
                "Debug",
                "Default",
                "PartialEq",
                "Eq",
                "PartialOrd",
                "Ord",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            // For other types, query the crate info
            _ => {
                let (crate_name, type_name) = if type_path.contains("::") {
                    self.parse_path(type_path)?
                } else {
                    ("self", type_path)
                };

                let crate_name = crate_name.to_string();

                // Try cache first
                if let Some(cached) = self.cache.get(&crate_name) {
                    return Ok(cached
                        .trait_implementations
                        .get(type_name)
                        .map(|traits| traits.iter().cloned().collect())
                        .unwrap_or_default());
                }

                // Query dynamically
                for querier in &mut self.queriers {
                    if querier.supports_crate(&crate_name) {
                        match querier.query_crate(&crate_name) {
                            Ok(crate_info) => {
                                let result = crate_info
                                    .trait_implementations
                                    .get(type_name)
                                    .map(|traits| traits.iter().cloned().collect())
                                    .unwrap_or_default();
                                // Cache the result
                                self.cache.insert(crate_name, crate_info);
                                return Ok(result);
                            }
                            Err(_) => continue, // Try next querier
                        }
                    }
                }

                Vec::new() // Default to empty if we can't find info
            }
        };

        Ok(traits)
    }

    pub fn parse_path<'a>(&self, path: &'a str) -> Result<(&'a str, &'a str), RustInteropError> {
        // Parse paths like "std::vec::Vec::new" -> ("std", "vec::Vec::new")
        let parts: Vec<&str> = path.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Err(RustInteropError::ParseError(format!(
                "Invalid path format: {}",
                path
            )));
        }
        Ok((parts[0], parts[1]))
    }
}

impl Default for DynamicRustRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// syn-based querier - Phase 2 implementation for comprehensive source parsing
#[derive(Debug)]
pub struct SynQuerier {
    cargo_metadata: Option<cargo_metadata::Metadata>,
    source_cache: HashMap<PathBuf, syn::File>,
}

impl SynQuerier {
    pub fn new(manifest_path: Option<PathBuf>) -> Result<Self, RustInteropError> {
        let metadata = if let Some(path) = manifest_path {
            cargo_metadata::MetadataCommand::new()
                .manifest_path(path)
                .exec()
                .map_err(|e| {
                    RustInteropError::CargoError(format!("Failed to get cargo metadata: {}", e))
                })?
        } else {
            cargo_metadata::MetadataCommand::new().exec().map_err(|e| {
                RustInteropError::CargoError(format!("Failed to get cargo metadata: {}", e))
            })?
        };

        Ok(Self {
            cargo_metadata: Some(metadata),
            source_cache: HashMap::new(),
        })
    }

    /// Extract crate information by parsing source files with syn
    pub fn extract_from_source(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        let metadata = self.cargo_metadata.as_ref().ok_or_else(|| {
            RustInteropError::CargoError("No cargo metadata available".to_string())
        })?;

        // Find the package
        let package = metadata
            .packages
            .iter()
            .find(|p| p.name == crate_name)
            .ok_or_else(|| RustInteropError::CrateNotFound(crate_name.to_string()))?;

        let mut crate_info = CrateInfo {
            name: package.name.clone(),
            version: package.version.to_string(),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        };

        // Collect all paths to parse to avoid borrowing issues
        let mut paths_to_parse = Vec::new();

        // Add library target if it exists
        if let Some(lib_target) = package
            .targets
            .iter()
            .find(|t| t.crate_types.contains(&"lib".to_string()))
        {
            paths_to_parse.push(lib_target.src_path.as_std_path().to_path_buf());
        }

        // Add binary targets
        for bin_target in package
            .targets
            .iter()
            .filter(|t| t.crate_types.contains(&"bin".to_string()))
        {
            paths_to_parse.push(bin_target.src_path.as_std_path().to_path_buf());
        }

        // Parse all collected files
        for path in paths_to_parse {
            self.parse_rust_file(&path, &mut crate_info)?;
        }

        Ok(crate_info)
    }

    fn parse_rust_file(
        &mut self,
        file_path: &std::path::Path,
        crate_info: &mut CrateInfo,
    ) -> Result<(), RustInteropError> {
        // Check cache first
        if let Some(parsed_file) = self.source_cache.get(file_path) {
            self.extract_items_from_file(parsed_file, crate_info)?;
            return Ok(());
        }

        // Read and parse the file
        let content = fs::read_to_string(file_path).map_err(|e| {
            RustInteropError::IoError(format!("Failed to read {}: {}", file_path.display(), e))
        })?;

        let parsed_file = syn::parse_file(&content).map_err(|e| {
            RustInteropError::ParseError(format!("Failed to parse {}: {}", file_path.display(), e))
        })?;

        // Cache the parsed file
        self.source_cache
            .insert(file_path.to_path_buf(), parsed_file.clone());

        // Extract items
        self.extract_items_from_file(&parsed_file, crate_info)?;

        Ok(())
    }

    fn extract_items_from_file(
        &self,
        file: &syn::File,
        crate_info: &mut CrateInfo,
    ) -> Result<(), RustInteropError> {
        for item in &file.items {
            match item {
                syn::Item::Fn(item_fn) => {
                    let function_info = self.parse_function(item_fn)?;
                    crate_info
                        .functions
                        .insert(function_info.full_path.clone(), function_info);
                }
                syn::Item::Struct(item_struct) => {
                    let type_info = self.parse_struct(item_struct)?;
                    crate_info
                        .types
                        .insert(type_info.full_path.clone(), type_info);
                }
                syn::Item::Enum(item_enum) => {
                    let type_info = self.parse_enum(item_enum)?;
                    crate_info
                        .types
                        .insert(type_info.full_path.clone(), type_info);
                }
                syn::Item::Trait(item_trait) => {
                    let trait_info = self.parse_trait(item_trait)?;
                    crate_info
                        .traits
                        .insert(trait_info.full_path.clone(), trait_info);
                }
                syn::Item::Impl(item_impl) => {
                    self.parse_impl_block(item_impl, crate_info)?;
                }
                _ => {} // Skip other items for now
            }
        }

        Ok(())
    }

    pub fn parse_function(&self, item_fn: &syn::ItemFn) -> Result<FunctionInfo, RustInteropError> {
        let name = item_fn.sig.ident.to_string();
        let full_path = name.clone(); // Simplified - would need module path resolution

        let generics = item_fn
            .sig
            .generics
            .params
            .iter()
            .filter_map(|param| {
                if let syn::GenericParam::Type(type_param) = param {
                    Some(GenericParam {
                        name: type_param.ident.to_string(),
                        bounds: type_param
                            .bounds
                            .iter()
                            .filter_map(|bound| {
                                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                    Some(quote::quote!(#trait_bound).to_string())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        default: type_param
                            .default
                            .as_ref()
                            .map(|default| quote::quote!(#default).to_string()),
                    })
                } else {
                    None
                }
            })
            .collect();

        let parameters = item_fn
            .sig
            .inputs
            .iter()
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input {
                    let name = if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        pat_ident.ident.to_string()
                    } else {
                        "param".to_string()
                    };

                    Some(Parameter {
                        name,
                        param_type: self.convert_syn_type_to_signature(&pat_type.ty),
                    })
                } else {
                    None
                }
            })
            .collect();

        let return_type = match &item_fn.sig.output {
            syn::ReturnType::Default => RustTypeSignature {
                raw: "()".to_string(),
                parsed: Some(RustType::Unit),
                lifetimes: vec![],
                bounds: vec![],
            },
            syn::ReturnType::Type(_, ty) => self.convert_syn_type_to_signature(ty),
        };

        Ok(FunctionInfo {
            name,
            full_path,
            generics,
            parameters,
            return_type,
            is_unsafe: item_fn.sig.unsafety.is_some(),
            is_const: item_fn.sig.constness.is_some(),
            documentation: self.extract_doc_comments(&item_fn.attrs),
        })
    }

    pub fn parse_struct(
        &self,
        item_struct: &syn::ItemStruct,
    ) -> Result<TypeInfo, RustInteropError> {
        let name = item_struct.ident.to_string();
        let full_path = name.clone(); // Simplified

        let generics = self.extract_generics(&item_struct.generics);

        let fields = match &item_struct.fields {
            syn::Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .map(|field| FieldInfo {
                    name: field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_default(),
                    field_type: self.convert_syn_type_to_signature(&field.ty),
                    is_public: matches!(field.vis, syn::Visibility::Public(_)),
                })
                .collect(),
            syn::Fields::Unnamed(fields_unnamed) => fields_unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| FieldInfo {
                    name: i.to_string(),
                    field_type: self.convert_syn_type_to_signature(&field.ty),
                    is_public: matches!(field.vis, syn::Visibility::Public(_)),
                })
                .collect(),
            syn::Fields::Unit => vec![],
        };

        Ok(TypeInfo {
            name,
            full_path,
            kind: TypeKind::Struct,
            generics,
            methods: vec![], // Will be filled by impl blocks
            fields,
            variants: vec![],
        })
    }

    pub fn parse_enum(&self, item_enum: &syn::ItemEnum) -> Result<TypeInfo, RustInteropError> {
        let name = item_enum.ident.to_string();
        let full_path = name.clone(); // Simplified

        let generics = self.extract_generics(&item_enum.generics);

        let variants = item_enum
            .variants
            .iter()
            .map(|variant| {
                let fields = match &variant.fields {
                    syn::Fields::Named(fields_named) => {
                        fields_named
                            .named
                            .iter()
                            .map(|field| {
                                FieldInfo {
                                    name: field
                                        .ident
                                        .as_ref()
                                        .map(|i| i.to_string())
                                        .unwrap_or_default(),
                                    field_type: self.convert_syn_type_to_signature(&field.ty),
                                    is_public: true, // Enum variant fields are always public
                                }
                            })
                            .collect()
                    }
                    syn::Fields::Unnamed(fields_unnamed) => fields_unnamed
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, field)| FieldInfo {
                            name: i.to_string(),
                            field_type: self.convert_syn_type_to_signature(&field.ty),
                            is_public: true,
                        })
                        .collect(),
                    syn::Fields::Unit => vec![],
                };

                VariantInfo {
                    name: variant.ident.to_string(),
                    fields,
                }
            })
            .collect();

        Ok(TypeInfo {
            name,
            full_path,
            kind: TypeKind::Enum,
            generics,
            methods: vec![],
            fields: vec![],
            variants,
        })
    }

    pub fn parse_trait(&self, item_trait: &syn::ItemTrait) -> Result<TraitInfo, RustInteropError> {
        let name = item_trait.ident.to_string();
        let full_path = name.clone(); // Simplified

        let methods = item_trait
            .items
            .iter()
            .filter_map(|item| {
                if let syn::TraitItem::Fn(method) = item {
                    self.parse_trait_method(method).ok()
                } else {
                    None
                }
            })
            .collect();

        let associated_types = item_trait
            .items
            .iter()
            .filter_map(|item| {
                if let syn::TraitItem::Type(type_item) = item {
                    Some(type_item.ident.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(TraitInfo {
            name,
            full_path,
            methods,
            associated_types,
        })
    }

    fn parse_trait_method(
        &self,
        method: &syn::TraitItemFn,
    ) -> Result<MethodInfo, RustInteropError> {
        let name = method.sig.ident.to_string();

        let self_kind = self.determine_self_kind(&method.sig.inputs);
        let generics = self.extract_generics(&method.sig.generics);

        let parameters = method
            .sig
            .inputs
            .iter()
            .skip(if self_kind == SelfKind::None { 0 } else { 1 }) // Skip self parameter
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input {
                    let name = if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        pat_ident.ident.to_string()
                    } else {
                        "param".to_string()
                    };

                    Some(Parameter {
                        name,
                        param_type: self.convert_syn_type_to_signature(&pat_type.ty),
                    })
                } else {
                    None
                }
            })
            .collect();

        let return_type = match &method.sig.output {
            syn::ReturnType::Default => RustTypeSignature {
                raw: "()".to_string(),
                parsed: Some(RustType::Unit),
                lifetimes: vec![],
                bounds: vec![],
            },
            syn::ReturnType::Type(_, ty) => self.convert_syn_type_to_signature(ty),
        };

        Ok(MethodInfo {
            name,
            self_kind,
            generics,
            parameters,
            return_type,
            is_unsafe: method.sig.unsafety.is_some(),
            is_const: method.sig.constness.is_some(),
        })
    }

    pub fn parse_impl_block(
        &self,
        item_impl: &syn::ItemImpl,
        crate_info: &mut CrateInfo,
    ) -> Result<(), RustInteropError> {
        // Extract the type name from the impl block
        let type_name = if let syn::Type::Path(type_path) = item_impl.self_ty.as_ref() {
            type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default()
        } else {
            return Ok(()); // Skip complex types for now
        };

        // Check if this is a trait implementation
        if let Some((_, trait_path, _)) = &item_impl.trait_ {
            // Extract trait name
            let trait_name = if let Some(segment) = trait_path.segments.last() {
                segment.ident.to_string()
            } else {
                return Ok(());
            };

            // Track the trait implementation
            crate_info
                .trait_implementations
                .entry(type_name.clone())
                .or_insert_with(HashSet::new)
                .insert(trait_name);
        }

        // Parse methods in the impl block
        for item in &item_impl.items {
            if let syn::ImplItem::Fn(method) = item {
                let method_info = self.parse_impl_method(method)?;

                // Add method to the corresponding type
                if let Some(type_info) = crate_info.types.get_mut(&type_name) {
                    type_info.methods.push(method_info);
                }
            }
        }

        Ok(())
    }

    fn parse_impl_method(&self, method: &syn::ImplItemFn) -> Result<MethodInfo, RustInteropError> {
        let name = method.sig.ident.to_string();

        let self_kind = self.determine_self_kind(&method.sig.inputs);
        let generics = self.extract_generics(&method.sig.generics);

        let parameters = method
            .sig
            .inputs
            .iter()
            .skip(if self_kind == SelfKind::None { 0 } else { 1 }) // Skip self parameter
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input {
                    let name = if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        pat_ident.ident.to_string()
                    } else {
                        "param".to_string()
                    };

                    Some(Parameter {
                        name,
                        param_type: self.convert_syn_type_to_signature(&pat_type.ty),
                    })
                } else {
                    None
                }
            })
            .collect();

        let return_type = match &method.sig.output {
            syn::ReturnType::Default => RustTypeSignature {
                raw: "()".to_string(),
                parsed: Some(RustType::Unit),
                lifetimes: vec![],
                bounds: vec![],
            },
            syn::ReturnType::Type(_, ty) => self.convert_syn_type_to_signature(ty),
        };

        Ok(MethodInfo {
            name,
            self_kind,
            generics,
            parameters,
            return_type,
            is_unsafe: method.sig.unsafety.is_some(),
            is_const: method.sig.constness.is_some(),
        })
    }

    pub fn determine_self_kind(
        &self,
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) -> SelfKind {
        if let Some(first_arg) = inputs.first() {
            match first_arg {
                syn::FnArg::Receiver(receiver) => {
                    if receiver.mutability.is_some() {
                        SelfKind::MutRef
                    } else if receiver.reference.is_some() {
                        SelfKind::Ref
                    } else {
                        SelfKind::Value
                    }
                }
                _ => SelfKind::None,
            }
        } else {
            SelfKind::None
        }
    }

    fn extract_generics(&self, generics: &syn::Generics) -> Vec<GenericParam> {
        generics
            .params
            .iter()
            .filter_map(|param| {
                if let syn::GenericParam::Type(type_param) = param {
                    Some(GenericParam {
                        name: type_param.ident.to_string(),
                        bounds: type_param
                            .bounds
                            .iter()
                            .filter_map(|bound| {
                                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                    Some(quote::quote!(#trait_bound).to_string())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        default: type_param
                            .default
                            .as_ref()
                            .map(|default| quote::quote!(#default).to_string()),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn convert_syn_type_to_signature(&self, ty: &syn::Type) -> RustTypeSignature {
        let raw = quote::quote!(#ty).to_string();
        let parsed = self.convert_syn_type_to_rust_type(ty);

        RustTypeSignature {
            raw,
            parsed,
            lifetimes: vec![], // TODO: Extract lifetimes properly
            bounds: vec![],
        }
    }

    pub fn convert_syn_type_to_rust_type(&self, ty: &syn::Type) -> Option<RustType> {
        match ty {
            syn::Type::Path(type_path) => {
                let path = &type_path.path;
                if path.segments.len() == 1 {
                    let segment = &path.segments[0];
                    match segment.ident.to_string().as_str() {
                        "i32" => Some(RustType::I32),
                        "i64" => Some(RustType::I64),
                        "isize" => Some(RustType::ISize),
                        "u32" => Some(RustType::U32),
                        "u64" => Some(RustType::U64),
                        "usize" => Some(RustType::USize),
                        "bool" => Some(RustType::Bool),
                        "char" => Some(RustType::Char),
                        "str" => Some(RustType::Str),
                        "String" => Some(RustType::String),
                        name => Some(RustType::Custom {
                            name: name.to_string(),
                            generics: vec![], // TODO: Extract generic arguments
                        }),
                    }
                } else {
                    Some(RustType::Custom {
                        name: quote::quote!(#path).to_string(),
                        generics: vec![],
                    })
                }
            }
            syn::Type::Reference(type_ref) => {
                let inner = self.convert_syn_type_to_rust_type(&type_ref.elem)?;
                if type_ref.mutability.is_some() {
                    Some(RustType::MutRef {
                        lifetime: type_ref.lifetime.as_ref().map(|lt| lt.ident.to_string()),
                        inner: Box::new(inner),
                    })
                } else {
                    Some(RustType::Ref {
                        lifetime: type_ref.lifetime.as_ref().map(|lt| lt.ident.to_string()),
                        inner: Box::new(inner),
                    })
                }
            }
            syn::Type::Tuple(type_tuple) => {
                if type_tuple.elems.is_empty() {
                    Some(RustType::Unit)
                } else {
                    None // TODO: Handle non-unit tuples
                }
            }
            _ => None,
        }
    }

    fn extract_doc_comments(&self, attrs: &[syn::Attribute]) -> Option<String> {
        let doc_comments: Vec<String> = attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc") {
                    if let syn::Meta::NameValue(meta_name_value) = &attr.meta {
                        if let syn::Expr::Lit(expr_lit) = &meta_name_value.value {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                }
                None
            })
            .collect();

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments.join("\n"))
        }
    }
}

impl RustQuerier for SynQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        self.extract_from_source(crate_name)
    }

    fn supports_crate(&self, _crate_name: &str) -> bool {
        self.cargo_metadata.is_some()
    }

    fn priority(&self) -> u32 {
        80 // Lower than rustdoc but higher than rust-analyzer
    }
}
