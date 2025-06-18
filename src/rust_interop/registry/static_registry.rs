//! Static registry for managing known Rust interop items.
//!
//! Provides registration and lookup of Rust functions and methods.

use super::dynamic_registry::DynamicRustRegistry;
use crate::error::VeltranoError;
use crate::rust_interop::{
    cache::*, parser::RustTypeParser, types::*, utils::camel_to_snake_case, ExternItem,
    RustInteropError,
};
use std::collections::HashMap;

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

        // format! macro
        self.register(ExternItem::Function {
            name: "format".to_string(),
            _path: "std::format!".to_string(),
            _params: vec![(
                "format".to_string(),
                RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                },
            )],
            _return_type: RustType::String,
            _is_unsafe: false,
        });

        // String methods
        self.register(ExternItem::Method {
            type_name: "String".to_string(),
            method_name: "new".to_string(),
            _self_kind: SelfKind::None, // static method
            _params: vec![],
            _return_type: RustType::String,
            _is_unsafe: false,
        });

        self.register(ExternItem::Method {
            type_name: "String".to_string(),
            method_name: "push_str".to_string(),
            _self_kind: SelfKind::MutRef,
            _params: vec![(
                "s".to_string(),
                RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                },
            )],
            _return_type: RustType::Unit,
            _is_unsafe: false,
        });

        // Vec methods
        self.register(ExternItem::Method {
            type_name: "Vec".to_string(),
            method_name: "new".to_string(),
            _self_kind: SelfKind::None, // static method
            _params: vec![],
            _return_type: RustType::Custom {
                name: "Vec".to_string(),
                generics: vec![RustType::Generic("T".to_string())],
            },
            _is_unsafe: false,
        });

        self.register(ExternItem::Method {
            type_name: "Vec".to_string(),
            method_name: "push".to_string(),
            _self_kind: SelfKind::MutRef,
            _params: vec![("value".to_string(), RustType::Generic("T".to_string()))],
            _return_type: RustType::Unit,
            _is_unsafe: false,
        });

        self.register(ExternItem::Method {
            type_name: "Vec".to_string(),
            method_name: "len".to_string(),
            _self_kind: SelfKind::Ref,
            _params: vec![],
            _return_type: RustType::USize,
            _is_unsafe: false,
        });

        // Option methods
        self.register(ExternItem::Method {
            type_name: "Option".to_string(),
            method_name: "is_some".to_string(),
            _self_kind: SelfKind::Ref,
            _params: vec![],
            _return_type: RustType::Bool,
            _is_unsafe: false,
        });

        self.register(ExternItem::Method {
            type_name: "Option".to_string(),
            method_name: "unwrap".to_string(),
            _self_kind: SelfKind::Value,
            _params: vec![],
            _return_type: RustType::Generic("T".to_string()),
            _is_unsafe: false,
        });

        // Result methods
        self.register(ExternItem::Method {
            type_name: "Result".to_string(),
            method_name: "is_ok".to_string(),
            _self_kind: SelfKind::Ref,
            _params: vec![],
            _return_type: RustType::Bool,
            _is_unsafe: false,
        });

        self.register(ExternItem::Method {
            type_name: "Result".to_string(),
            method_name: "unwrap".to_string(),
            _self_kind: SelfKind::Value,
            _params: vec![],
            _return_type: RustType::Generic("T".to_string()),
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
            } => format!("{}::{}", type_name, method_name),
            ExternItem::_Type { name, .. } => format!("type::{}", name),
        };
        self.items.insert(key, item);
    }

    #[allow(dead_code)]
    pub fn lookup_function(&self, name: &str) -> Option<&ExternItem> {
        self.items.get(name)
    }

    #[allow(dead_code)]
    pub fn lookup_method(&self, type_name: &str, method_name: &str) -> Option<&ExternItem> {
        let key = format!("{}::{}", type_name, method_name);
        self.items.get(&key)
    }

    /// Query for type information dynamically
    #[allow(dead_code)]
    pub fn query_type(&mut self, type_path: &str) -> Option<TypeInfo> {
        match self.dynamic_registry.get_type(type_path) {
            Ok(type_info) => type_info,
            Err(_) => None,
        }
    }

    /// Query for imported method information
    #[allow(dead_code)]
    pub fn query_imported_method(&mut self, method_path: &str) -> Option<ImportedMethodInfo> {
        // Parse method path like "std::vec::Vec::push"
        let parts: Vec<&str> = method_path.split("::").collect();
        if parts.len() < 2 {
            return None;
        }

        let method_name = parts.last()?;
        let type_path = parts[..parts.len() - 1].join("::");

        // Query type info
        let type_info = self.query_type(&type_path)?;

        // Find method in type
        let method = type_info.methods.iter().find(|m| m.name == *method_name)?;

        // Build ImportedMethodInfo
        Some(ImportedMethodInfo {
            _method_name: method.name.clone(),
            self_kind: method.self_kind.clone(),
            _parameters: method
                .parameters
                .iter()
                .filter_map(|p| p.param_type.parsed.clone())
                .collect(),
            return_type: method.return_type.parsed.clone().unwrap_or(RustType::Unit),
            _trait_name: None,
        })
    }

    /// Check if a type implements a specific trait
    pub fn type_implements_trait(
        &mut self,
        rust_type: &RustType,
        trait_name: &str,
    ) -> Result<bool, VeltranoError> {
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

    /// Query method signature dynamically from crate metadata
    /// This integrates with the DynamicRustRegistry for full method resolution
    pub fn query_method_signature(
        &mut self,
        rust_type: &RustType,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, VeltranoError> {
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

    /// Get method information for a type by querying available methods
    /// This includes both inherent methods and trait methods
    fn get_method_info(&self, _type_path: &str, _method_name: &str) -> Option<ImportedMethodInfo> {
        // No hardcoded method signatures - rely entirely on dynamic registry
        None
    }

    /// Query method signature using the dynamic registry system
    fn query_dynamic_method_signature(
        &mut self,
        type_path: &str,
        method_name: &str,
    ) -> Result<Option<ImportedMethodInfo>, VeltranoError> {
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
    ) -> Result<Option<ImportedMethodInfo>, VeltranoError> {
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
