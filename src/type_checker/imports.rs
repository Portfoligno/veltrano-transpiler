//! Import handling for the type checker
//!
//! This module contains import-related types and logic for managing
//! method imports and resolving them during type checking.

use crate::ast::{CallExpr, ImportStmt};
use crate::error::{SourceLocation, Span};
use crate::rust_interop::{RustInteropRegistry, RustType, RustTypeParser, SelfKind};
use crate::types::VeltranoType;
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
    /// User imports - these take precedence
    user_imports: HashMap<String, Vec<ImportedMethod>>,
    /// Built-in imports - only used if no user imports exist for a method
    builtin_imports: HashMap<String, Vec<ImportedMethod>>,
    /// Import validation errors collected during checking
    import_errors: Vec<TypeCheckError>,
}

impl ImportHandler {
    /// Create a new import handler
    pub fn new() -> Self {
        Self {
            user_imports: HashMap::new(),
            builtin_imports: HashMap::new(),
            import_errors: Vec::new(),
        }
    }

    /// Get any import errors that were collected
    pub fn get_import_errors(&self) -> &[TypeCheckError] {
        &self.import_errors
    }

    /// Register an invalid import for better error messages at use site
    fn register_invalid_import(&mut self, key: String, import: &ImportStmt) {
        // Store as a trait import so it can still be found at use site
        self.user_imports
            .entry(key)
            .or_insert_with(Vec::new)
            .push(ImportedMethod::TraitMethod {
                trait_name: import.type_name.clone(),
                method_name: import.method_name.clone(),
            });
    }

    /// Get imports for a given method name - user imports shadow built-ins completely
    pub fn get_imports(&self, name: &str) -> Option<Vec<ImportedMethod>> {
        // Check user imports first
        if let Some(user_methods) = self.user_imports.get(name) {
            return Some(user_methods.clone());
        }

        // Fall back to built-in imports only if no user imports exist
        self.builtin_imports.get(name).cloned()
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

        // First check if it's a known trait
        if trait_checker.trait_exists(&import.type_name) {
            // Valid trait import
            self.user_imports.entry(key).or_insert_with(Vec::new).push(
                ImportedMethod::TraitMethod {
                    trait_name: import.type_name.clone(),
                    method_name: import.method_name.clone(),
                },
            );
            return Ok(());
        }

        // If not a trait, try to parse as a type and check if the method exists on that type
        if let Ok(rust_type) = RustTypeParser::parse(&import.type_name) {
            crate::debug_println!(
                "DEBUG: Parsed import type '{}' as {:?}",
                import.type_name,
                rust_type
            );
            // Check if this type has the requested method
            match trait_checker.query_method_signature(&rust_type, &import.method_name) {
                Ok(Some(_)) => {
                    crate::debug_println!(
                        "DEBUG: Found method '{}' on type {:?}, storing as TypeMethod",
                        import.method_name,
                        rust_type
                    );
                    // This is a valid type-based import
                    self.user_imports
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
                }
                Ok(None) => {
                    // Type exists but method doesn't
                    crate::debug_println!(
                        "DEBUG: Method '{}' not found on type {:?}",
                        import.method_name,
                        rust_type
                    );
                    self.import_errors.push(TypeCheckError::InvalidImport {
                        type_name: import.type_name.clone(),
                        method_name: import.method_name.clone(),
                        reason: format!(
                            "Type '{}' has no method '{}'",
                            import.type_name, import.method_name
                        ),
                        location: import.location.clone(),
                    });
                    // Still register it for better error at use site
                    self.register_invalid_import(key, import);
                    return Ok(());
                }
                Err(_) => {
                    // Error querying, fall through to trait check
                    crate::debug_println!(
                        "DEBUG: Error querying method signature for {:?}.{}",
                        rust_type,
                        import.method_name
                    );
                }
            }
        }

        // Neither a valid trait nor a type with the method
        self.import_errors.push(TypeCheckError::InvalidImport {
            type_name: import.type_name.clone(),
            method_name: import.method_name.clone(),
            reason: format!("Type or trait '{}' not found", import.type_name),
            location: import.location.clone(),
        });
        // Still register it for better error at use site
        self.register_invalid_import(key, import);

        Ok(())
    }

    /// Check if we have any imports (user or built-in) for a method name
    #[allow(dead_code)]
    pub fn has_imports(&self, name: &str) -> bool {
        self.user_imports.contains_key(name) || self.builtin_imports.contains_key(name)
    }

    /// Check a standalone method call using imports (e.g., Vec.new())
    pub fn check_standalone_method_call(
        &self,
        func_name: &str,
        call: &CallExpr,
        trait_checker: &mut RustInteropRegistry,
        method_resolutions: &mut HashMap<usize, MethodResolution>,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        let imports =
            self.get_imports(func_name)
                .ok_or_else(|| TypeCheckError::FunctionNotFound {
                    name: func_name.to_string(),
                    location: SourceLocation::new(span.start_line(), span.start_column()),
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
                    location: SourceLocation::new(span.start_line(), span.start_column()),
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
                    location: SourceLocation::new(span.start_line(), span.start_column()),
                })
            }
        }
    }
}

/// Register built-in imports at type checker initialization
pub fn register_builtin_imports(
    handler: &mut ImportHandler,
    trait_checker: &mut RustInteropRegistry,
) {
    crate::debug_println!("DEBUG: Registering built-in imports");

    // Clone trait
    register_trait_method(handler, "Clone", "clone", "clone", trait_checker);

    // ToString trait (register with camelCase so it converts properly to snake_case)
    register_trait_method(handler, "ToString", "toString", "toString", trait_checker);

    // Length methods (multiple registrations with aliasing)
    register_type_method(handler, "Vec", "len", "length", trait_checker);
    register_type_method(handler, "String", "len", "length", trait_checker);
    register_type_method(handler, "str", "len", "length", trait_checker);

    // Slice conversion
    register_type_method(handler, "Vec", "asSlice", "toSlice", trait_checker);

    crate::debug_println!(
        "DEBUG: Built-in imports registered. Has clone: {}, Has toString: {}, Has length: {}",
        handler.has_imports("clone"),
        handler.has_imports("toString"),
        handler.has_imports("length")
    );
}

/// Helper to register a type method as a built-in import
fn register_type_method(
    handler: &mut ImportHandler,
    type_name: &str,
    rust_method: &str,
    veltrano_name: &str,
    _trait_checker: &mut RustInteropRegistry,
) {
    // Built-in imports are always valid, so we store them directly
    let key = veltrano_name.to_string();
    let rust_type = RustTypeParser::parse(type_name).expect("Built-in type should parse");

    handler
        .builtin_imports
        .entry(key)
        .or_insert_with(Vec::new)
        .push(ImportedMethod::TypeMethod {
            rust_type,
            method_name: rust_method.to_string(),
        });

    crate::debug_println!(
        "DEBUG: Registered built-in type method {}.{} as {}",
        type_name,
        rust_method,
        veltrano_name
    );
}

/// Helper to register a trait method as a built-in import
fn register_trait_method(
    handler: &mut ImportHandler,
    trait_name: &str,
    rust_method: &str,
    veltrano_name: &str,
    _trait_checker: &mut RustInteropRegistry,
) {
    // Built-in imports are always valid, so we store them directly
    let key = veltrano_name.to_string();

    handler
        .builtin_imports
        .entry(key)
        .or_insert_with(Vec::new)
        .push(ImportedMethod::TraitMethod {
            trait_name: trait_name.to_string(),
            method_name: rust_method.to_string(),
        });

    crate::debug_println!(
        "DEBUG: Registered built-in trait method {}.{} as {}",
        trait_name,
        rust_method,
        veltrano_name
    );
}
