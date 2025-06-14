//! Module for handling Rust interoperability and type signature extraction
//! This provides mechanisms to:
//! 1. Declare external Rust functions/methods
//! 2. Parse Rust type signatures
//! 3. Convert between Rust and Veltrano type representations
//! 4. Dynamically query Rust toolchain for type information

mod cache;
mod compiler;
mod types;

pub use cache::{
    CrateInfo, FunctionInfo, GenericParam, MethodInfo, Parameter, RustTypeSignature,
    TraitInfo, TypeInfo, TypeKind,
};
pub use compiler::{RustdocQuerier, SynQuerier};
pub use types::{ImportedMethodInfo, RustType, SelfKind};

use std::collections::{HashMap, HashSet};

/// Convert camelCase to snake_case for Rust naming conventions
pub fn camel_to_snake_case(name: &str) -> String {
    let mut result = String::new();

    for ch in name.chars() {
        if ch == '_' {
            // Underscore becomes double underscore
            result.push_str("__");
        } else if ch.is_uppercase() {
            // Uppercase becomes underscore + lowercase
            result.push('_');
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            // Lowercase stays as is
            result.push(ch);
        }
    }

    result
}

/// Represents an external Rust item (function, method, or type)
#[derive(Debug, Clone)]
pub enum ExternItem {
    Function {
        name: String,
        _path: String, // Full Rust path e.g., "std::vec::Vec::new"
        _params: Vec<(String, RustType)>,
        _return_type: RustType,
        _is_unsafe: bool,
    },
    Method {
        type_name: String,
        method_name: String,
        _self_kind: SelfKind,
        _params: Vec<(String, RustType)>,
        _return_type: RustType,
        _is_unsafe: bool,
    },
    _Type {
        name: String,
        rust_path: String,
        generic_params: Vec<String>,
    },
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
            _path: "std::println!".to_string(),
            _params: vec![(
                "format".to_string(),
                RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                },
            )],
            _return_type: RustType::Unit,
            _is_unsafe: false,
        });

        // Vec::new
        self.register(ExternItem::Method {
            type_name: "Vec".to_string(),
            method_name: "new".to_string(),
            _self_kind: SelfKind::None,
            _params: vec![],
            _return_type: RustType::Vec(Box::new(RustType::Generic("T".to_string()))),
            _is_unsafe: false,
        });

        // String::from
        self.register(ExternItem::Method {
            type_name: "String".to_string(),
            method_name: "from".to_string(),
            _self_kind: SelfKind::None,
            _params: vec![(
                "s".to_string(),
                RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                },
            )],
            _return_type: RustType::String,
            _is_unsafe: false,
        });

        // Vec::push
        self.register(ExternItem::Method {
            type_name: "Vec".to_string(),
            method_name: "push".to_string(),
            _self_kind: SelfKind::MutRef,
            _params: vec![("value".to_string(), RustType::Generic("T".to_string()))],
            _return_type: RustType::Unit,
            _is_unsafe: false,
        });

        // String::len
        self.register(ExternItem::Method {
            type_name: "String".to_string(),
            method_name: "len".to_string(),
            _self_kind: SelfKind::Ref,
            _params: vec![],
            _return_type: RustType::USize,
            _is_unsafe: false,
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
            ExternItem::_Type { name, .. } => format!("type::{}", name),
        };
        self.items.insert(key, item);
    }

    /// Check if a type implements a specific trait
    pub fn type_implements_trait(
        &mut self,
        rust_type: &RustType,
        trait_name: &str,
    ) -> Result<bool, RustInteropError> {
        // Convert to string only at the lowest level
        let type_path = rust_type.to_rust_syntax();

        // For built-in types, we can have hardcoded knowledge
        let implements = match type_path.as_str() {
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
                matches!(
                    trait_name,
                    "Clone" | "Debug" | "Display" | "ToString" | "Into"
                )
            }
            "&str" | "str" => {
                matches!(trait_name, "Debug" | "Display" | "ToString" | "Into")
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
        rust_type: &RustType,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, RustInteropError> {
        // Try method resolution following Rust's rules
        // First try the exact type, then try dereferenced types
        let type_sequence = self.build_method_resolution_sequence(rust_type);

        for candidate_type in type_sequence {
            // Convert to string only at the lowest level for CrateInfo query
            let type_path = candidate_type.to_rust_syntax();
            crate::debug_println!(
                "DEBUG: query_method_signature - trying candidate_type: {:?} -> type_path: {}",
                candidate_type,
                type_path
            );

            // First try hardcoded method info for built-in types
            if let Some(method_info) = self.get_method_info(&type_path, method_name) {
                return Ok(Some(method_info));
            }

            // For other types, use the dynamic registry to query method signatures
            if let result @ Ok(Some(_)) =
                self.query_dynamic_method_signature(&type_path, method_name)
            {
                return result;
            }
        }

        Ok(None)
    }

    /// Build the sequence of types to check for method resolution
    /// Following Rust's method resolution order
    fn build_method_resolution_sequence(&self, rust_type: &RustType) -> Vec<RustType> {
        let mut sequence = Vec::new();

        // For references, check the reference type first (for impl Clone for &T)
        // Then check the inner type (for impl Clone for T)
        match rust_type {
            RustType::Ref { inner, lifetime: _ } => {
                // First check &T itself
                sequence.push(rust_type.clone());
                // Then check T (the compiler will auto-ref if needed)
                sequence.push(inner.as_ref().clone());
            }
            _ => {
                // For non-references, just check the type itself
                sequence.push(rust_type.clone());
            }
        }

        sequence
    }

    /// Query method signature using the dynamic registry system
    fn query_dynamic_method_signature(
        &mut self,
        type_path: &str,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, RustInteropError> {
        // Convert Veltrano method name (camelCase) to Rust method name (snake_case)
        let rust_method_name = camel_to_snake_case(method_name);
        crate::debug_println!("DEBUG: query_dynamic_method_signature - type_path: {}, method_name: {} -> rust_method_name: {}", type_path, method_name, rust_method_name);

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
                if method.name == rust_method_name {
                    return Ok(Some(ImportedMethodInfo {
                        _method_name: method_name.to_string(), // Keep original Veltrano name
                        self_kind: method.self_kind.clone(),
                        _parameters: self.convert_parameters(&method.parameters),
                        return_type: self.convert_rust_type_signature(&method.return_type),
                        _trait_name: None, // Inherent method
                    }));
                }
            }
        }

        // If not found in inherent methods, search trait methods
        self.query_trait_method_signature(type_path, method_name)
    }

    /// Check if a type is a standard library type
    fn is_stdlib_type(&self, type_path: &str) -> bool {
        matches!(
            type_path,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "str"
                | "String"
                | "()"
                | "Vec"
                | "Option"
                | "Result"
        )
    }

    /// Convert method parameters from the dynamic registry format
    fn convert_parameters(&self, parameters: &[Parameter]) -> Vec<RustType> {
        parameters
            .iter()
            .filter_map(|param| param.param_type.parsed.clone())
            .collect()
    }

    /// Convert return type from the dynamic registry format
    fn convert_rust_type_signature(&self, return_type: &RustTypeSignature) -> RustType {
        return_type.parsed.clone().unwrap_or(RustType::Unit)
    }

    /// Query trait method signature for a type
    fn query_trait_method_signature(
        &mut self,
        type_path: &str,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, RustInteropError> {
        // Convert Veltrano method name (camelCase) to Rust method name (snake_case)
        let rust_method_name = camel_to_snake_case(method_name);
        crate::debug_println!("DEBUG: query_trait_method_signature - type_path: {}, method_name: {} -> rust_method_name: {}", type_path, method_name, rust_method_name);

        // Special handling for reference types
        // In Rust, &T automatically has certain trait implementations based on T
        let (actual_type_path, traits) =
            if type_path.starts_with("&") && !type_path.starts_with("&&") {
                let inner_type = &type_path[1..]; // Remove the & prefix

                // For Clone trait on &T, check if T implements Clone
                if rust_method_name == "clone" {
                    // Check if the inner type implements Clone
                    if let Ok(rust_type) = RustTypeParser::parse(inner_type) {
                        if self.type_implements_trait(&rust_type, "Clone")? {
                            // &T implements Clone when T: Clone
                            // Return the traits with Clone included
                            let mut inner_traits =
                                self.dynamic_registry.get_implemented_traits(inner_type)?;
                            if !inner_traits.contains(&"Clone".to_string()) {
                                inner_traits.push("Clone".to_string());
                            }
                            (type_path, inner_traits)
                        } else {
                            // T doesn't implement Clone, so &T doesn't have clone()
                            (type_path, vec![])
                        }
                    } else {
                        // Couldn't parse inner type, fall back to normal query
                        (
                            type_path,
                            self.dynamic_registry.get_implemented_traits(type_path)?,
                        )
                    }
                } else {
                    // For other methods on &T, use normal trait lookup
                    (
                        type_path,
                        self.dynamic_registry.get_implemented_traits(type_path)?,
                    )
                }
            } else {
                // Not a reference type, use normal lookup
                (
                    type_path,
                    self.dynamic_registry.get_implemented_traits(type_path)?,
                )
            };

        crate::debug_println!(
            "DEBUG: query_trait_method_signature - found traits for {}: {:?}",
            actual_type_path,
            traits
        );

        // Into trait is special - it has a generic parameter T that determines the return type
        // For &str.into(), we need to figure out what T is from context
        if method_name == "into" && traits.contains(&"Into".to_string()) {
            crate::debug_println!(
                "DEBUG: query_trait_method_signature - Special handling for Into trait on {}",
                actual_type_path
            );
            // For now, we'll let the trait query proceed normally
            // The type checker will need to handle the generic return type
        }

        // Search each trait for the method
        for trait_name in traits {
            let trait_path = format!("std::{}", trait_name); // Assuming std library traits
            crate::debug_println!(
                "DEBUG: query_trait_method_signature - checking trait_path: {}",
                trait_path
            );

            if let Ok(Some(trait_info)) = self.dynamic_registry.get_trait(&trait_path) {
                crate::debug_println!(
                    "DEBUG: query_trait_method_signature - found trait_info for {}",
                    trait_path
                );
                for method in &trait_info.methods {
                    crate::debug_println!(
                        "DEBUG: query_trait_method_signature - checking method {} against {}",
                        method.name,
                        rust_method_name
                    );
                    if method.name == rust_method_name {
                        crate::debug_println!(
                            "DEBUG: query_trait_method_signature - FOUND method {} in trait {}!",
                            rust_method_name,
                            trait_name
                        );
                        // Found the method in this trait
                        return Ok(Some(ImportedMethodInfo {
                            _method_name: method_name.to_string(), // Keep original Veltrano name
                            self_kind: method.self_kind.clone(),
                            _parameters: self.convert_parameters(&method.parameters),
                            return_type: if method.return_type.raw == "Self" {
                                crate::debug_println!("DEBUG: query_trait_method_signature - return type is Self for {}", actual_type_path);
                                // For trait methods returning Self, return the concrete type
                                // Special case: &T.clone() returns T, not &T
                                if rust_method_name == "clone" && actual_type_path.starts_with("&")
                                {
                                    // Clone on &T returns T
                                    let inner_type = &actual_type_path[1..];
                                    match inner_type {
                                        "String" => RustType::String,
                                        "i32" => RustType::I32,
                                        "i64" => RustType::I64,
                                        "isize" => RustType::ISize,
                                        "u32" => RustType::U32,
                                        "u64" => RustType::U64,
                                        "usize" => RustType::USize,
                                        "bool" => RustType::Bool,
                                        "char" => RustType::Char,
                                        _ => RustType::Custom {
                                            name: inner_type.to_string(),
                                            generics: vec![],
                                        },
                                    }
                                } else {
                                    // Normal Self resolution
                                    match actual_type_path {
                                        "String" => RustType::String,
                                        "i32" => RustType::I32,
                                        "i64" => RustType::I64,
                                        "isize" => RustType::ISize,
                                        "u32" => RustType::U32,
                                        "u64" => RustType::U64,
                                        "usize" => RustType::USize,
                                        "bool" => RustType::Bool,
                                        "char" => RustType::Char,
                                        _ => RustType::Custom {
                                            name: actual_type_path.to_string(),
                                            generics: vec![],
                                        },
                                    }
                                }
                            } else {
                                crate::debug_println!("DEBUG: query_trait_method_signature - return type is NOT Self, raw: {}", method.return_type.raw);
                                self.convert_rust_type_signature(&method.return_type)
                            },
                            _trait_name: Some(trait_name.clone()),
                        }));
                    }
                }
            } else {
                crate::debug_println!(
                    "DEBUG: query_trait_method_signature - trait_info NOT found for {}",
                    trait_path
                );
            }
        }

        Ok(None)
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

        // Handle Result<T, E>
        if let Some(inner) = trimmed
            .strip_prefix("Result<")
            .and_then(|s| s.strip_suffix('>'))
        {
            // Split by comma, but need to handle nested generics
            let mut depth = 0;
            let mut split_pos = None;
            for (i, ch) in inner.chars().enumerate() {
                match ch {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    ',' if depth == 0 => {
                        split_pos = Some(i);
                        break;
                    }
                    _ => {}
                }
            }

            if let Some(pos) = split_pos {
                let ok_type = inner[..pos].trim();
                let err_type = inner[pos + 1..].trim();
                return Ok(RustType::Result {
                    ok: Box::new(Self::parse(ok_type)?),
                    err: Box::new(Self::parse(err_type)?),
                });
            } else {
                return Err("Invalid Result type: missing error type".to_string());
            }
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
    CrateNotFound(String),
}

impl std::fmt::Display for RustInteropError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RustInteropError::CargoError(msg) => write!(f, "Cargo error: {}", msg),
            RustInteropError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            RustInteropError::IoError(msg) => write!(f, "IO error: {}", msg),
            RustInteropError::CrateNotFound(name) => write!(f, "Crate not found: {}", name),
        }
    }
}

impl std::error::Error for RustInteropError {}

/// Trait for querying Rust type information
pub trait RustQuerier: std::fmt::Debug {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError>;
    fn supports_crate(&self, crate_name: &str) -> bool;
    fn priority(&self) -> u32; // Higher priority queriers tried first
}

/// Standard library querier with minimal hardcoded knowledge
/// This will be replaced with proper rustdoc parsing in the future
#[derive(Debug)]
pub struct StdLibQuerier {
    cache: Option<CrateInfo>,
}

impl StdLibQuerier {
    pub fn new() -> Self {
        Self { cache: None }
    }

    fn create_std_crate_info(&self) -> CrateInfo {
        let mut crate_info = CrateInfo {
            name: "std".to_string(),
            version: "1.0.0".to_string(),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        };

        // Add Clone trait
        let clone_trait = TraitInfo {
            name: "Clone".to_string(),
            full_path: "std::clone::Clone".to_string(),
            methods: vec![MethodInfo {
                name: "clone".to_string(),
                self_kind: SelfKind::Ref,
                generics: vec![],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "Self".to_string(),
                    parsed: None, // Self type
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            associated_types: vec![],
        };
        // Store with just "Clone" since that's what will be looked up after parse_path("std::Clone")
        crate_info.traits.insert("Clone".to_string(), clone_trait);

        // Add ToString trait
        let to_string_trait = TraitInfo {
            name: "ToString".to_string(),
            full_path: "std::string::ToString".to_string(),
            methods: vec![MethodInfo {
                name: "to_string".to_string(),
                self_kind: SelfKind::Ref,
                generics: vec![],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "String".to_string(),
                    parsed: Some(RustType::String),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            associated_types: vec![],
        };
        // Store with just "ToString" since that's what will be looked up after parse_path("std::ToString")
        crate_info
            .traits
            .insert("ToString".to_string(), to_string_trait);

        // Add Into trait
        let into_trait = TraitInfo {
            name: "Into".to_string(),
            full_path: "std::convert::Into".to_string(),
            methods: vec![MethodInfo {
                name: "into".to_string(),
                self_kind: SelfKind::Value,
                generics: vec![GenericParam {
                    name: "T".to_string(),
                    bounds: vec![],
                    default: None,
                }],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "T".to_string(),
                    parsed: Some(RustType::Generic("T".to_string())),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            associated_types: vec![],
        };
        crate_info.traits.insert("Into".to_string(), into_trait);

        // Add trait implementations for common types
        for typ in &[
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64", "bool", "char",
        ] {
            let mut traits = HashSet::new();
            traits.insert("Clone".to_string());
            traits.insert("ToString".to_string());
            crate_info
                .trait_implementations
                .insert(typ.to_string(), traits);
        }

        // String implements Clone, ToString, and Into
        let mut string_traits = HashSet::new();
        string_traits.insert("Clone".to_string());
        string_traits.insert("ToString".to_string());
        string_traits.insert("Into".to_string());
        crate_info
            .trait_implementations
            .insert("String".to_string(), string_traits);

        // &str implements ToString and Into
        let mut str_traits = HashSet::new();
        str_traits.insert("ToString".to_string());
        str_traits.insert("Into".to_string());
        crate_info
            .trait_implementations
            .insert("&str".to_string(), str_traits);

        // Add Vec type with methods
        let vec_type = TypeInfo {
            name: "Vec".to_string(),
            full_path: "std::vec::Vec".to_string(),
            kind: TypeKind::Struct,
            generics: vec![GenericParam {
                name: "T".to_string(),
                bounds: vec![],
                default: None,
            }],
            methods: vec![
                MethodInfo {
                    name: "new".to_string(),
                    self_kind: SelfKind::None, // Static method
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "Vec<T>".to_string(),
                        parsed: Some(RustType::Vec(Box::new(RustType::Generic("T".to_string())))),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "push".to_string(),
                    self_kind: SelfKind::MutRef,
                    generics: vec![],
                    parameters: vec![Parameter {
                        name: "value".to_string(),
                        param_type: RustTypeSignature {
                            raw: "T".to_string(),
                            parsed: Some(RustType::Generic("T".to_string())),
                            lifetimes: vec![],
                            bounds: vec![],
                        },
                    }],
                    return_type: RustTypeSignature {
                        raw: "()".to_string(),
                        parsed: Some(RustType::Unit),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "len".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "usize".to_string(),
                        parsed: Some(RustType::USize),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
            ],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("Vec".to_string(), vec_type);

        // Vec implements Clone
        let mut vec_traits = HashSet::new();
        vec_traits.insert("Clone".to_string());
        crate_info
            .trait_implementations
            .insert("Vec".to_string(), vec_traits);

        // Add numeric type methods (i64 as example)
        let i64_type = TypeInfo {
            name: "i64".to_string(),
            full_path: "i64".to_string(),
            kind: TypeKind::Struct, // Primitive types are treated as structs
            generics: vec![],
            methods: vec![MethodInfo {
                name: "abs".to_string(),
                self_kind: SelfKind::Value, // Takes self by value
                generics: vec![],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "i64".to_string(),
                    parsed: Some(RustType::I64),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("i64".to_string(), i64_type);

        // Add String methods
        let string_type = TypeInfo {
            name: "String".to_string(),
            full_path: "std::string::String".to_string(),
            kind: TypeKind::Struct,
            generics: vec![],
            methods: vec![
                MethodInfo {
                    name: "len".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "usize".to_string(),
                        parsed: Some(RustType::USize),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "chars".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "Chars".to_string(),
                        parsed: Some(RustType::Custom {
                            name: "Chars".to_string(),
                            generics: vec![],
                        }),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "clone".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "String".to_string(),
                        parsed: Some(RustType::String),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
            ],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("String".to_string(), string_type);

        crate_info
    }
}

impl RustQuerier for StdLibQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        if crate_name == "std" {
            if self.cache.is_none() {
                self.cache = Some(self.create_std_crate_info());
            }
            Ok(self.cache.as_ref().unwrap().clone())
        } else {
            Err(RustInteropError::CrateNotFound(crate_name.to_string()))
        }
    }

    fn supports_crate(&self, crate_name: &str) -> bool {
        crate_name == "std"
    }

    fn priority(&self) -> u32 {
        200 // Higher priority than rustdoc and syn queriers
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
        // First add the standard library querier with built-in knowledge
        registry.add_querier(Box::new(StdLibQuerier::new()));

        // Then add general purpose queriers
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

    pub fn _get_function(&mut self, path: &str) -> Result<Option<FunctionInfo>, RustInteropError> {
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

    /// Get trait information by path
    pub fn get_trait(&mut self, path: &str) -> Result<Option<TraitInfo>, RustInteropError> {
        let (crate_name, trait_path) = self.parse_path(path)?;
        let crate_name = crate_name.to_string();
        let trait_path = trait_path.to_string();

        // Try cache first
        if let Some(cached) = self.cache.get(&crate_name) {
            return Ok(cached.traits.get(&trait_path).cloned());
        }

        // Query dynamically
        for querier in &mut self.queriers {
            if querier.supports_crate(&crate_name) {
                match querier.query_crate(&crate_name) {
                    Ok(crate_info) => {
                        let result = crate_info.traits.get(&trait_path).cloned();
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
    pub fn _type_implements_trait(
        &mut self,
        type_path: &str,
        trait_name: &str,
    ) -> Result<bool, RustInteropError> {
        // For built-in types, delegate to RustInteropRegistry
        let mut basic_registry = RustInteropRegistry::new();

        // Try to parse the type path into a RustType
        if let Ok(rust_type) = RustTypeParser::parse(type_path) {
            let basic_result = basic_registry.type_implements_trait(&rust_type, trait_name);
            if basic_result.is_ok() {
                return basic_result;
            }
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
                "ToString",
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
                "Into", // All types implement Into<Self>
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
