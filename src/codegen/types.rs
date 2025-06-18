//! Type generation with ownership handling.
//!
//! Converts Veltrano types to Rust types with proper lifetimes.

use crate::types::{VeltranoType, TypeConstructor};
use super::CodeGenerator;

impl CodeGenerator {
    /// Generate Rust type representation from Veltrano type annotation
    pub(super) fn generate_type(&mut self, type_annotation: &VeltranoType) {

        // For data class types that need lifetime parameters, we need special handling
        if let TypeConstructor::Custom(name) = &type_annotation.constructor {
            if self.data_classes_with_lifetime.contains(name) {
                // For naturally referenced custom types with lifetime parameters
                self.output.push('&');
                if self.generating_bump_function {
                    self.output.push_str("'a ");
                }
                self.output.push_str(name);
                if self.generating_bump_function {
                    self.output.push_str("<'a>");
                }
                return;
            }
        }

        // Use the new to_rust_type_with_lifetime method
        let lifetime = if self.generating_bump_function {
            Some("'a".to_string())
        } else {
            None
        };

        let rust_type =
            type_annotation.to_rust_type_with_lifetime(&mut self.trait_checker, lifetime);
        self.output.push_str(&rust_type.to_rust_syntax());
    }

    /// Check if a type needs lifetime parameters (is naturally referenced)
    pub(super) fn type_needs_lifetime(&mut self, veltrano_type: &VeltranoType) -> bool {

        match &veltrano_type.constructor {
            // Reference types always need lifetimes
            TypeConstructor::Ref | TypeConstructor::MutRef => true,
            // Use trait checking for base types
            _ if veltrano_type.args.is_empty() => {
                !veltrano_type.implements_copy(&mut self.trait_checker)
            }
            // Composed types need further analysis
            _ => false,
        }
    }

    /// Generate type annotation for data class fields (with lifetime 'a)
    pub(super) fn generate_data_class_field_type(&mut self, type_annotation: &VeltranoType) {

        // For data class fields, we always need lifetime 'a for reference types
        // Special handling for custom types with lifetime parameters
        if let TypeConstructor::Custom(name) = &type_annotation.constructor {
            if self.data_classes_with_lifetime.contains(name) {
                self.output.push_str("&'a ");
                self.output.push_str(name);
                self.output.push_str("<'a>");
                return;
            }
        }

        // For owned custom types in data class fields
        if let TypeConstructor::Own = &type_annotation.constructor {
            if let Some(inner) = type_annotation.inner() {
                if let TypeConstructor::Custom(name) = &inner.constructor {
                    self.output.push_str(name);
                    if self.data_classes_with_lifetime.contains(name) {
                        self.output.push_str("<'a>");
                    }
                    return;
                }
            }
        }

        // For all other types, use the standard conversion with lifetime 'a
        let rust_type = type_annotation
            .to_rust_type_with_lifetime(&mut self.trait_checker, Some("'a".to_string()));
        self.output.push_str(&rust_type.to_rust_syntax());
    }
}
