//! Type checking error types and error analysis
//!
//! This module contains the error types for the type checker
//! and methods for error analysis and suggestions.

use crate::ast::BinaryOp;
use crate::types::SourceLocation;
use crate::types::{TypeConstructor, VeltranoType};

/// Type checking errors with detailed information
#[derive(Debug)]
pub enum TypeCheckError {
    TypeMismatch {
        expected: VeltranoType,
        actual: VeltranoType,
        location: SourceLocation,
    },
    TypeMismatchWithSuggestion {
        expected: VeltranoType,
        actual: VeltranoType,
        location: SourceLocation,
        suggestion: String,
    },
    MethodNotFound {
        receiver_type: VeltranoType,
        method: String,
        location: SourceLocation,
    },
    MethodNotFoundWithSuggestion {
        receiver_type: VeltranoType,
        method: String,
        location: SourceLocation,
        suggestion: String,
    },
    FieldNotFound {
        object_type: VeltranoType,
        field: String,
        location: SourceLocation,
    },
    FieldNotFoundWithSuggestion {
        object_type: VeltranoType,
        field: String,
        location: SourceLocation,
        suggestion: String,
    },
    ArgumentCountMismatch {
        function: String,
        expected: usize,
        actual: usize,
        location: SourceLocation,
    },
    _IndexingNotSupported {
        object_type: VeltranoType,
        index_type: VeltranoType,
        location: SourceLocation,
    },
    _BinaryOperatorNotSupported {
        operator: BinaryOp,
        left_type: VeltranoType,
        right_type: VeltranoType,
        location: SourceLocation,
    },
    VariableNotFound {
        name: String,
        location: SourceLocation,
    },
    FunctionNotFound {
        name: String,
        location: SourceLocation,
    },
    AmbiguousMethodCall {
        method: String,
        receiver_type: VeltranoType,
        candidates: Vec<String>,
        location: SourceLocation,
    },
    InvalidTypeConstructor {
        message: String,
        location: SourceLocation,
    },
    UnsupportedFeature {
        feature: String,
        location: SourceLocation,
    },
    _InvalidType {
        type_name: String,
        reason: String,
        location: SourceLocation,
    },
    _InvalidImport {
        type_name: String,
        method_name: String,
        location: SourceLocation,
    },
}

/// Information about a resolved method call
#[derive(Debug, Clone)]
pub struct MethodResolution {
    pub rust_type: crate::rust_interop::RustType,
    pub method_name: String,
}

/// Error analysis and suggestion generation
pub struct ErrorAnalyzer;

impl ErrorAnalyzer {
    /// Enhance an error with helpful suggestions
    pub fn enhance_error(error: TypeCheckError) -> TypeCheckError {
        match error {
            TypeCheckError::TypeMismatch {
                expected,
                actual,
                location,
            } => {
                if let Some(suggestion) = Self::suggest_type_conversion(&expected, &actual) {
                    TypeCheckError::TypeMismatchWithSuggestion {
                        expected,
                        actual,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::TypeMismatch {
                        expected,
                        actual,
                        location,
                    }
                }
            }
            TypeCheckError::MethodNotFound {
                receiver_type,
                method,
                location,
            } => {
                if let Some(suggestion) = Self::suggest_method_conversion(&receiver_type, &method) {
                    TypeCheckError::MethodNotFoundWithSuggestion {
                        receiver_type,
                        method,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::MethodNotFound {
                        receiver_type,
                        method,
                        location,
                    }
                }
            }
            TypeCheckError::FieldNotFound {
                object_type,
                field,
                location,
            } => {
                if let Some(suggestion) = Self::suggest_field_conversion(&object_type, &field) {
                    TypeCheckError::FieldNotFoundWithSuggestion {
                        object_type,
                        field,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::FieldNotFound {
                        object_type,
                        field,
                        location,
                    }
                }
            }
            // Pass through other error types unchanged
            other => other,
        }
    }

    /// Suggest conversion from actual type to expected type
    fn suggest_type_conversion(expected: &VeltranoType, actual: &VeltranoType) -> Option<String> {
        // Handle common conversion patterns with new type system

        // Pattern 1: Own<T> to T (remove ownership) -> .ref()
        if actual.constructor == TypeConstructor::Own {
            if let Some(inner) = actual.inner() {
                if inner == expected {
                    return Some(".ref()".to_string());
                }
            }
        }

        // Pattern 2: MutRef<T> to Ref<T> -> .ref()
        if expected.constructor == TypeConstructor::Ref
            && actual.constructor == TypeConstructor::MutRef
        {
            if let (Some(expected_inner), Some(actual_inner)) = (expected.inner(), actual.inner()) {
                if expected_inner == actual_inner {
                    return Some(".ref()".to_string());
                }
            }
        }

        // Pattern 5: Vec<T> to slice conversion -> .toSlice()
        if actual.constructor == TypeConstructor::Vec
            && expected.constructor == TypeConstructor::Ref
        {
            if let (Some(expected_inner), Some(actual_inner)) = (expected.inner(), actual.inner()) {
                if expected_inner == actual_inner {
                    return Some(".toSlice()".to_string());
                }
            }
        }

        // Pattern 6: Own<Vec<T>> to slice -> .ref().toSlice()
        if expected.constructor == TypeConstructor::Ref
            && actual.constructor == TypeConstructor::Own
        {
            if let Some(actual_inner) = actual.inner() {
                if actual_inner.constructor == TypeConstructor::Vec {
                    if let (Some(expected_inner), Some(vec_inner)) =
                        (expected.inner(), actual_inner.inner())
                    {
                        if expected_inner == vec_inner {
                            return Some(".ref().toSlice()".to_string());
                        }
                    }
                }
            }
        }

        // Pattern 7: Array to slice conversion
        if expected.constructor == TypeConstructor::Ref {
            if let TypeConstructor::Array(_) = actual.constructor {
                if let (Some(expected_inner), Some(actual_inner)) =
                    (expected.inner(), actual.inner())
                {
                    if expected_inner == actual_inner {
                        return Some(".toSlice()".to_string());
                    }
                }
            }
        }

        // Pattern 8: Own<Array<T, N>> to slice -> .ref().toSlice()
        if expected.constructor == TypeConstructor::Ref
            && actual.constructor == TypeConstructor::Own
        {
            if let Some(actual_inner) = actual.inner() {
                if let TypeConstructor::Array(_) = actual_inner.constructor {
                    if let (Some(expected_inner), Some(array_inner)) =
                        (expected.inner(), actual_inner.inner())
                    {
                        if expected_inner == array_inner {
                            return Some(".ref().toSlice()".to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Suggest method call with proper ownership conversion
    fn suggest_method_conversion(receiver_type: &VeltranoType, method: &str) -> Option<String> {
        // Common pattern: owned types need .ref() before calling borrowed methods
        if receiver_type.constructor == TypeConstructor::Own {
            // Suggest adding .ref() before the method call for owned types
            Some(format!(".ref().{}()", method))
        } else {
            None
        }
    }

    /// Suggest field access with proper ownership conversion
    fn suggest_field_conversion(object_type: &VeltranoType, _field: &str) -> Option<String> {
        match &object_type.constructor {
            TypeConstructor::Custom(class_name) => {
                // For reference types (Person, not Own<Person>), suggest using an owned value
                // Without access to data class definitions, we can't verify if the field exists
                // So we provide a generic suggestion for custom types
                Some(format!(
                    "Field access requires an owned value (Own<{}>)",
                    class_name
                ))
            }
            TypeConstructor::Own => {
                // For owned types, field access should work directly
                // If we're here, it means the field doesn't exist
                None
            }
            _ => None,
        }
    }
}
