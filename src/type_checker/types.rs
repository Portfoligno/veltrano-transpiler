//! Type validation and type-related utilities for the type checker
//!
//! This module contains type validation logic and helper functions
//! for working with VeltranoType in the type checking context.

use crate::error::SourceLocation;
use crate::rust_interop::RustInteropRegistry;
use crate::types::{TypeConstructor, VeltranoType};

use super::error::TypeCheckError;

/// Type validation logic
pub struct TypeValidator;

impl TypeValidator {
    /// Validate a type, ensuring it's well-formed
    pub fn validate_type(
        veltrano_type: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Result<(), TypeCheckError> {
        match &veltrano_type.constructor {
            TypeConstructor::Own => {
                // Validate Own<T> type constructor
                if let Some(inner) = veltrano_type.inner() {
                    // First validate the inner type recursively
                    Self::validate_type(inner, trait_checker)?;

                    // Then validate the Own<T> constraint
                    if let Err(err_msg) = validate_own_constructor(inner, trait_checker) {
                        return Err(TypeCheckError::InvalidTypeConstructor {
                            message: err_msg,
                            // TODO: VeltranoType doesn't carry location information.
                            // This would require adding Located<VeltranoType> or similar.
                            location: SourceLocation::new(1, 1),
                        });
                    }
                } else {
                    return Err(TypeCheckError::InvalidTypeConstructor {
                        message: "Own<T> requires a type parameter".to_string(),
                        // TODO: VeltranoType doesn't carry location information.
                        location: SourceLocation::new(1, 1),
                    });
                }
            }
            _ => {
                // For other type constructors, recursively validate type arguments
                for arg in &veltrano_type.args {
                    Self::validate_type(arg, trait_checker)?;
                }
            }
        }
        Ok(())
    }

    /// Check if two types are equal (no implicit conversions)
    pub fn types_equal(a: &VeltranoType, b: &VeltranoType) -> bool {
        a == b // Simple structural equality
    }
}

/// Validate if Own<T> type constructor is valid with the given inner type
pub fn validate_own_constructor(
    inner: &VeltranoType,
    trait_checker: &mut RustInteropRegistry,
) -> Result<(), String> {
    // Check if the inner type implements Copy (is naturally owned)
    let is_copy = inner.implements_copy(trait_checker);

    if is_copy {
        return Err(format!(
            "Cannot use Own<{:?}>. Types that implement Copy are always owned by default and don't need the Own<> wrapper.",
            inner.constructor
        ));
    }

    // Check for specific invalid combinations
    match &inner.constructor {
        TypeConstructor::MutRef => {
            Err("Cannot use Own<MutRef<T>>. MutRef<T> is already owned.".to_string())
        }
        TypeConstructor::Own => {
            Err("Cannot use Own<Own<T>>. This creates double ownership.".to_string())
        }
        _ => Ok(()),
    }
}

/// Substitute a generic type parameter with a concrete type
pub fn substitute_generic_type(
    type_template: &VeltranoType,
    param_name: &str,
    concrete_type: &VeltranoType,
) -> VeltranoType {
    match &type_template.constructor {
        TypeConstructor::Generic(name, _) if name == param_name => {
            // Replace the generic parameter with the concrete type
            concrete_type.clone()
        }
        _ => {
            // Recursively substitute in type arguments
            let substituted_args = type_template
                .args
                .iter()
                .map(|arg| substitute_generic_type(arg, param_name, concrete_type))
                .collect();

            VeltranoType {
                constructor: type_template.constructor.clone(),
                args: substituted_args,
            }
        }
    }
}
