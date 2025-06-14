//! Import handling for the type checker
//!
//! This module contains import-related types and logic for managing
//! method imports and resolving them during type checking.

use crate::ast::{CallExpr, ImportStmt};
use crate::rust_interop::{RustInteropRegistry, RustType, RustTypeParser, SelfKind};
use crate::types::{SourceLocation, VeltranoType};
use std::collections::HashMap;

use super::error::{MethodResolution, TypeCheckError};

/// Represents an imported method, which can come from either a type or a trait
#[derive(Debug, Clone)]
pub enum ImportedMethod {
    /// Import from a specific type: String.len
    TypeMethod {
        rust_type: RustType,
        method_name: String,
    },
    /// Import from a trait: Into.into
    TraitMethod {
        trait_name: String,
        method_name: String,
    },
}

/// Import handler for managing and resolving method imports
pub struct ImportHandler {
    /// Maps method names/aliases to their imported methods
    imports: HashMap<String, Vec<ImportedMethod>>,
}

impl ImportHandler {
    /// Create a new import handler
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    /// Get imports for a given method name
    pub fn get_imports(&self, name: &str) -> Option<Vec<ImportedMethod>> {
        self.imports.get(name).cloned()
    }

    /// Check import statement and register it for method resolution
    pub fn check_import_statement(
        &mut self,
        import: &ImportStmt,
        trait_checker: &mut RustInteropRegistry,
    ) -> Result<(), TypeCheckError> {
        // Store the import for later method resolution, allowing multiple imports with same name
        let key = import
            .alias
            .clone()
            .unwrap_or_else(|| import.method_name.clone());

        // First, try to parse as a type and check if the method exists on that type
        if let Ok(rust_type) = RustTypeParser::parse(&import.type_name) {
            crate::debug_println!(
                "DEBUG: Parsed import type '{}' as {:?}",
                import.type_name,
                rust_type
            );
            // Check if this type has the requested method
            if let Ok(Some(_)) =
                trait_checker.query_method_signature(&rust_type, &import.method_name)
            {
                crate::debug_println!(
                    "DEBUG: Found method '{}' on type {:?}, storing as TypeMethod",
                    import.method_name,
                    rust_type
                );
                // This is a valid type-based import
                self.imports
                    .entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push(ImportedMethod::TypeMethod {
                        rust_type,
                        method_name: import.method_name.clone(),
                    });
                crate::debug_println!(
                    "DEBUG: Stored import with key '{}' (alias: {:?}, method: {})",
                    key,
                    import.alias,
                    import.method_name
                );
                return Ok(());
            } else {
                crate::debug_println!(
                    "DEBUG: Method '{}' not found on type {:?}",
                    import.method_name,
                    rust_type
                );
            }
        } else {
            crate::debug_println!(
                "DEBUG: Failed to parse '{}' as a Rust type",
                import.type_name
            );
        }

        // If it's not a valid type-based import, assume it's a trait import
        // We can't validate trait imports at import time because we don't know
        // what types will use them yet
        self.imports
            .entry(key)
            .or_insert_with(Vec::new)
            .push(ImportedMethod::TraitMethod {
                trait_name: import.type_name.clone(),
                method_name: import.method_name.clone(),
            });

        Ok(())
    }

    /// Check a standalone method call using imports (e.g., Vec.new())
    pub fn check_standalone_method_call(
        &self,
        func_name: &str,
        call: &CallExpr,
        trait_checker: &mut RustInteropRegistry,
        method_resolutions: &mut HashMap<usize, MethodResolution>,
    ) -> Result<VeltranoType, TypeCheckError> {
        let imports =
            self.get_imports(func_name)
                .ok_or_else(|| TypeCheckError::FunctionNotFound {
                    name: func_name.to_string(),
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        _column: 0,
                        _source_line: "".to_string(),
                    },
                })?;

        // For standalone method calls like Vec.new(), we need to check if the method
        // is a static method (takes no self parameter)
        crate::debug_println!(
            "DEBUG: Checking standalone method call '{}' with {} imports",
            func_name,
            imports.len()
        );

        let mut matching_imports = Vec::new();
        let mut candidate_descriptions = Vec::new();

        for import in &imports {
            match import {
                ImportedMethod::TypeMethod {
                    rust_type,
                    method_name,
                } => {
                    crate::debug_println!(
                        "DEBUG: Checking type import {:?}.{}",
                        rust_type,
                        method_name
                    );
                    if let Ok(Some(method_info)) =
                        trait_checker.query_method_signature(&rust_type, &method_name)
                    {
                        crate::debug_println!(
                            "DEBUG: Found method info, self_kind = {:?}",
                            method_info.self_kind
                        );
                        // Check if this is a static method (SelfKind::None)
                        if matches!(method_info.self_kind, SelfKind::None) {
                            // This is a static method, it can be called standalone
                            // TODO: Check arguments once we have argument support

                            // Convert the return type
                            if let Ok(return_type) = method_info.return_type.to_veltrano_type() {
                                crate::debug_println!(
                                    "DEBUG: Static method matched! Return type: {:?}",
                                    return_type
                                );
                                matching_imports.push((
                                    rust_type.clone(),
                                    method_name.clone(),
                                    return_type,
                                ));
                                candidate_descriptions
                                    .push(format!("{:?}.{}", rust_type, method_name));
                            }
                        } else {
                            // Record as candidate even if not static
                            candidate_descriptions.push(format!("{:?}.{}", rust_type, method_name));
                        }
                    }
                }
                ImportedMethod::TraitMethod {
                    trait_name,
                    method_name,
                } => {
                    // Trait methods typically aren't static, but we check anyway
                    crate::debug_println!(
                        "DEBUG: Checking trait import {}.{} for static method",
                        trait_name,
                        method_name
                    );
                    // For now, trait methods can't be called as standalone functions
                    // They need a receiver that implements the trait
                }
            }
        }

        // Check how many imports matched
        match matching_imports.len() {
            0 => {
                // No matching static method found
                Err(TypeCheckError::FunctionNotFound {
                    name: func_name.to_string(),
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        _column: 0,
                        _source_line: "".to_string(),
                    },
                })
            }
            1 => {
                // Exactly one import matched - use it
                let (rust_type, method_name, return_type) = &matching_imports[0];

                // Store the resolution for codegen
                method_resolutions.insert(
                    call.id,
                    MethodResolution {
                        rust_type: rust_type.clone(),
                        method_name: method_name.clone(),
                    },
                );

                Ok(return_type.clone())
            }
            _ => {
                // Multiple imports matched - ambiguous
                Err(TypeCheckError::AmbiguousMethodCall {
                    method: func_name.to_string(),
                    receiver_type: VeltranoType::unit(), // No receiver for standalone calls
                    candidates: candidate_descriptions,
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        _column: 0,
                        _source_line: "".to_string(),
                    },
                })
            }
        }
    }
}
