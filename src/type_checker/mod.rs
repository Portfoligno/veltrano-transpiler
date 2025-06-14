pub mod error;
mod expressions;
mod imports;
mod statements;
mod types;

use crate::ast::*;
use crate::builtins::BuiltinRegistry;
use crate::rust_interop::RustInteropRegistry;
use crate::types::*;

pub use error::{MethodResolution, TypeCheckError};
use imports::{ImportHandler, ImportedMethod};

/// Main type checker with strict type checking (no implicit conversions)
pub struct VeltranoTypeChecker {
    env: TypeEnvironment,
    trait_checker: RustInteropRegistry,
    builtin_registry: BuiltinRegistry,
    import_handler: ImportHandler,
    method_resolutions: std::collections::HashMap<usize, MethodResolution>, // Maps method call IDs to their resolutions
}

impl VeltranoTypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            env: TypeEnvironment::new(),
            trait_checker: RustInteropRegistry::new(),
            builtin_registry: BuiltinRegistry::new(),
            import_handler: ImportHandler::new(),
            method_resolutions: std::collections::HashMap::new(),
        };

        // Initialize built-in functions and methods
        checker.init_builtin_functions();
        checker
    }

    /// Get the method resolutions map for passing to codegen
    pub fn get_method_resolutions(&self) -> &std::collections::HashMap<usize, MethodResolution> {
        &self.method_resolutions
    }

    fn init_builtin_functions(&mut self) {
        // Register built-in function signatures from the builtin registry
        let function_signatures = self.builtin_registry.get_function_signatures();

        for signature in function_signatures {
            self.env.declare_function(signature.name.clone(), signature);
        }
    }

    /// Main entry point for type checking a program
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeCheckError>> {
        let mut errors = Vec::new();

        // First pass: collect all function signatures (including nested ones)
        for statement in &program.statements {
            if let Err(error) = self.collect_function_signatures_from_statement(statement) {
                errors.push(error);
            }
        }

        // Second pass: type check all statements
        for statement in &program.statements {
            if let Err(error) = self.check_statement(statement) {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check method call with expected type for generic inference
    fn check_method_call_with_expected_type(
        &mut self,
        method_call: &MethodCallExpr,
        expected_type: Option<&VeltranoType>,
    ) -> Result<VeltranoType, TypeCheckError> {
        let receiver_type = self.check_expression(&method_call.object)?;

        // Check if this method is explicitly imported - imports shadow built-ins completely
        if let Some(imports) = self.import_handler.get_imports(&method_call.method) {
            crate::debug_println!(
                "DEBUG: Checking method '{}' with {} imports and expected type: {:?}",
                method_call.method,
                imports.len(),
                expected_type
            );
            let mut matching_imports = Vec::new();
            let mut candidate_descriptions = Vec::new();

            // Check each imported function with this name
            for import in &imports {
                match import {
                    ImportedMethod::TypeMethod {
                        rust_type,
                        method_name,
                    } => {
                        crate::debug_println!(
                            "DEBUG: Checking type import {:?}.{} against receiver {:?}",
                            rust_type,
                            method_name,
                            receiver_type
                        );
                        // Try to typecheck this import with the receiver
                        match self.check_imported_method_call(
                            &receiver_type,
                            &rust_type,
                            &method_name,
                            method_call,
                        ) {
                            Ok(return_type) => {
                                crate::debug_println!(
                                    "DEBUG: Import matched! Storing for method call ID {}",
                                    method_call.id
                                );
                                matching_imports.push((
                                    rust_type.clone(),
                                    method_name.clone(),
                                    return_type,
                                ));
                                candidate_descriptions
                                    .push(format!("{:?}.{}", rust_type, method_name));
                            }
                            Err(_) => {
                                crate::debug_println!("DEBUG: Import didn't match");
                                // This import doesn't match, but we still record it as a candidate
                                candidate_descriptions
                                    .push(format!("{:?}.{}", rust_type, method_name));
                            }
                        }
                    }
                    ImportedMethod::TraitMethod {
                        trait_name,
                        method_name,
                    } => {
                        crate::debug_println!(
                            "DEBUG: Checking trait import {}.{} against receiver {:?}",
                            trait_name,
                            method_name,
                            receiver_type
                        );
                        // For trait imports, we need to check if the receiver implements the trait
                        let receiver_rust_type =
                            receiver_type.to_rust_type(&mut self.trait_checker);

                        // Check if the receiver type implements the trait
                        if let Ok(true) = self
                            .trait_checker
                            .type_implements_trait(&receiver_rust_type, trait_name)
                        {
                            // Get the method signature from the trait
                            if let Ok(Some(method_info)) = self
                                .trait_checker
                                .query_method_signature(&receiver_rust_type, method_name)
                            {
                                // Check if the receiver can provide the required access
                                if self
                                    .builtin_registry
                                    .receiver_can_provide_rust_access_for_imported(
                                        &receiver_type,
                                        &method_info.self_kind,
                                        &mut self.trait_checker,
                                    )
                                {
                                    // Handle generic return types with inference
                                    let return_type =
                                        if let crate::rust_interop::RustType::Generic(param_name) =
                                            &method_info.return_type
                                        {
                                            if let Some(expected) = expected_type {
                                                crate::debug_println!(
                                                    "DEBUG: Inferring generic parameter {} = {:?}",
                                                    param_name,
                                                    expected
                                                );
                                                // Use the expected type as the inferred type for the generic parameter
                                                expected.clone()
                                            } else {
                                                // No expected type, can't infer
                                                crate::debug_println!("DEBUG: Cannot infer generic parameter {} without expected type", param_name);
                                                match method_info.return_type.to_veltrano_type() {
                                                    Ok(t) => t,
                                                    Err(_) => continue, // Skip this method if we can't convert the type
                                                }
                                            }
                                        } else {
                                            match method_info.return_type.to_veltrano_type() {
                                                Ok(t) => t,
                                                Err(_) => continue, // Skip this method if we can't convert the type
                                            }
                                        };

                                    crate::debug_println!(
                                        "DEBUG: Trait import matched! Storing for method call ID {} with return type {:?}",
                                        method_call.id, return_type
                                    );
                                    // For trait methods, we store the trait name as the "type" for UFCS generation
                                    matching_imports.push((
                                        crate::rust_interop::RustType::Custom {
                                            name: trait_name.clone(),
                                            generics: vec![],
                                        },
                                        method_name.clone(),
                                        return_type,
                                    ));
                                    candidate_descriptions
                                        .push(format!("{}.{}", trait_name, method_name));
                                }
                            }
                        } else {
                            crate::debug_println!(
                                "DEBUG: Type doesn't implement trait {}",
                                trait_name
                            );
                            candidate_descriptions.push(format!("{}.{}", trait_name, method_name));
                        }
                    }
                }
            }

            // Check how many imports matched
            match matching_imports.len() {
                0 => {
                    // No imported methods matched, but imports exist - don't fall back to built-ins
                    return Err(TypeCheckError::MethodNotFound {
                        receiver_type,
                        method: method_call.method.clone(),
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            _column: 0,
                            _source_line: "".to_string(),
                        },
                    });
                }
                1 => {
                    // Exactly one import matched - store the resolution
                    let (rust_type, method_name, return_type) = &matching_imports[0];
                    crate::debug_println!(
                        "DEBUG: Storing method resolution for ID {}: {:?}.{}",
                        method_call.id,
                        rust_type,
                        method_name
                    );
                    let resolution = MethodResolution {
                        rust_type: rust_type.clone(),
                        method_name: method_name.clone(),
                    };
                    self.method_resolutions.insert(method_call.id, resolution);
                    return Ok(return_type.clone());
                }
                _ => {
                    // Multiple imports matched - ambiguous
                    return Err(TypeCheckError::AmbiguousMethodCall {
                        method: method_call.method.clone(),
                        receiver_type,
                        candidates: candidate_descriptions,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            _column: 0,
                            _source_line: "".to_string(),
                        },
                    });
                }
            }
        }

        // No imports, check built-ins
        self.check_builtin_method_call(&receiver_type, method_call)
    }

    /// Check method call expression
    fn check_method_call(
        &mut self,
        method_call: &MethodCallExpr,
    ) -> Result<VeltranoType, TypeCheckError> {
        self.check_method_call_with_expected_type(method_call, None)
    }

    /// Check built-in method call (when no imports exist)
    fn check_builtin_method_call(
        &mut self,
        receiver_type: &VeltranoType,
        method_call: &MethodCallExpr,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Only check built-ins if no imports exist for this method name
        if let Some(return_type) = self.builtin_registry.get_method_return_type(
            &method_call.method,
            &receiver_type,
            &mut self.trait_checker,
        ) {
            return Ok(return_type);
        }

        // Method not found in any system
        Err(TypeCheckError::MethodNotFound {
            receiver_type: receiver_type.clone(),
            method: method_call.method.clone(),
            location: SourceLocation {
                file: "unknown".to_string(),
                line: 0,
                _column: 0,
                _source_line: "".to_string(),
            },
        })
    }

    /// Check imported method call with full signature validation
    fn check_imported_method_call(
        &mut self,
        receiver_type: &VeltranoType,
        rust_type: &crate::rust_interop::RustType,
        rust_method_name: &str,
        _method_call: &MethodCallExpr,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Query the imported method signature first to know what self_kind it expects
        let method_info = if let Ok(Some(info)) = self
            .trait_checker
            .query_method_signature(rust_type, rust_method_name)
        {
            info
        } else {
            return Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: rust_method_name.to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    _column: 0,
                    _source_line: "".to_string(),
                },
            });
        };

        // Check if the receiver type matches what the method expects
        // based on the method's self_kind
        let receiver_matches = if let Ok(import_veltrano_type) = rust_type.to_veltrano_type() {
            crate::debug_println!(
                "DEBUG: Checking receiver match - rust_type: {:?} -> veltrano_type: {:?}",
                rust_type,
                import_veltrano_type
            );
            match method_info.self_kind {
                crate::rust_interop::SelfKind::Value => {
                    // Method expects self - for Copy types, allow bare type; otherwise Own<T>
                    if self
                        .trait_checker
                        .type_implements_trait(rust_type, "Copy")
                        .unwrap_or(false)
                    {
                        // Copy types can be used directly
                        receiver_type == &import_veltrano_type
                    } else {
                        // Non-Copy types must be wrapped in Own
                        // The import type might already be Own<T> from to_veltrano_type
                        if matches!(&import_veltrano_type.constructor, TypeConstructor::Own) {
                            receiver_type == &import_veltrano_type
                        } else {
                            matches!(&receiver_type.constructor, TypeConstructor::Own)
                                && receiver_type.inner() == Some(&import_veltrano_type)
                        }
                    }
                }
                crate::rust_interop::SelfKind::Ref => {
                    // Method expects &self
                    if self
                        .trait_checker
                        .type_implements_trait(rust_type, "Copy")
                        .unwrap_or(false)
                    {
                        // Copy types need Ref<Self>
                        matches!(&receiver_type.constructor, TypeConstructor::Ref)
                            && receiver_type.inner() == Some(&import_veltrano_type)
                    } else {
                        // Non-Copy types use bare Self
                        // But to_veltrano_type may have wrapped it in Own, so check both
                        if matches!(&import_veltrano_type.constructor, TypeConstructor::Own) {
                            // If import type is Own<T>, extract T for comparison
                            import_veltrano_type.inner() == Some(receiver_type)
                        } else {
                            receiver_type == &import_veltrano_type
                        }
                    }
                }
                crate::rust_interop::SelfKind::MutRef => {
                    // Method expects &mut self
                    if self
                        .trait_checker
                        .type_implements_trait(rust_type, "Copy")
                        .unwrap_or(false)
                    {
                        // Copy types need MutRef<Self>
                        matches!(&receiver_type.constructor, TypeConstructor::MutRef)
                            && receiver_type.inner() == Some(&import_veltrano_type)
                    } else {
                        // Non-Copy types use MutRef<Own<Self>>
                        if matches!(&receiver_type.constructor, TypeConstructor::MutRef) {
                            if let Some(inner) = receiver_type.inner() {
                                // inner should be Own<T> and import_veltrano_type should be Own<T>
                                inner == &import_veltrano_type
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                }
                crate::rust_interop::SelfKind::None => {
                    // Static method - shouldn't be called as method
                    false
                }
            }
        } else {
            false
        };

        if !receiver_matches {
            crate::debug_println!(
                "DEBUG: Type mismatch - receiver {:?}, self_kind {:?}",
                receiver_type,
                method_info.self_kind
            );
            return Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: rust_method_name.to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    _column: 0,
                    _source_line: "".to_string(),
                },
            });
        }

        // Check if the receiver type can provide the required access
        if !self
            .builtin_registry
            .receiver_can_provide_rust_access_for_imported(
                receiver_type,
                &method_info.self_kind,
                &mut self.trait_checker,
            )
        {
            return Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: rust_method_name.to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    _column: 0,
                    _source_line: "".to_string(),
                },
            });
        }

        // Convert the Rust return type to Veltrano type
        if let Ok(veltrano_return_type) = method_info.return_type.to_veltrano_type() {
            Ok(veltrano_return_type)
        } else {
            // Return a reasonable default type if conversion fails
            Ok(receiver_type.clone())
        }
    }
}
