//! Dynamic registry for querying Rust type information.
//!
//! Manages multiple queriers and provides a unified interface.

use crate::error::VeltranoError;
use crate::rust_interop::{
    rustdoc_querier::RustdocQuerier, stdlib_querier::StdLibQuerier, syn_querier::SynQuerier,
    CrateInfo, RustInteropError, RustQuerier, TypeInfo,
};
use crate::rust_interop::cache::{FunctionInfo, TraitInfo};
use std::collections::HashMap;

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

    pub fn _get_function(&mut self, path: &str) -> Result<Option<FunctionInfo>, VeltranoError> {
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

    pub fn get_type(&mut self, path: &str) -> Result<Option<TypeInfo>, VeltranoError> {
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
    pub fn get_trait(&mut self, path: &str) -> Result<Option<TraitInfo>, VeltranoError> {
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

    /// Get all traits implemented by a type
    pub fn get_implemented_traits(
        &mut self,
        type_path: &str,
    ) -> Result<Vec<String>, VeltranoError> {
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

    /// Parse a path like "std::vec::Vec::new" into (crate, item_path)
    pub fn parse_path<'a>(&self, path: &'a str) -> Result<(&'a str, &'a str), VeltranoError> {
        let parts: Vec<&str> = path.split("::").collect();
        if parts.len() < 2 {
            return Err(VeltranoError::from(RustInteropError::ParseError(
                "Path must have at least two segments".to_string(),
            )));
        }

        // First part is the crate name
        let crate_name = parts[0];

        Ok((crate_name, &path[crate_name.len() + 2..]))
    }

    /// Check if a type implements a trait
    pub fn _type_implements_trait(
        &mut self,
        type_name: &str,
        trait_name: &str,
    ) -> Result<bool, VeltranoError> {
        // Get the list of implemented traits
        let traits = self.get_implemented_traits(type_name)?;
        Ok(traits.contains(&trait_name.to_string()))
    }
}
