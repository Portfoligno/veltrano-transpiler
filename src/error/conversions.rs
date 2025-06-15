//! Conversion utilities for migrating existing error types to VeltranoError

use super::{ErrorKind, VeltranoError};
use crate::codegen::CodegenError;
use crate::rust_interop::RustInteropError;
use crate::type_checker::TypeCheckError;

/// Convert CodegenError to VeltranoError
impl From<CodegenError> for VeltranoError {
    fn from(err: CodegenError) -> Self {
        use super::Span;

        match err {
            CodegenError::InvalidDataClassSyntax {
                constructor,
                reason,
                location,
            } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!(
                    "Invalid data class syntax for '{}': {}",
                    constructor, reason
                ),
            )
            .with_span(Span::single(location)),
            CodegenError::InvalidShorthandUsage {
                field_name,
                context,
                location,
            } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!("Invalid shorthand usage of '{}' in {}", field_name, context),
            )
            .with_span(Span::single(location)),
            CodegenError::InvalidBuiltinArguments {
                builtin,
                reason,
                location,
            } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!("Invalid arguments for builtin '{}': {}", builtin, reason),
            )
            .with_span(Span::single(location)),
            CodegenError::MissingImport {
                method,
                type_name,
                location,
            } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!(
                    "Method '{}' requires import for type '{}'",
                    method, type_name
                ),
            )
            .with_span(Span::single(location)),
        }
    }
}

/// Convert RustInteropError to VeltranoError
impl From<RustInteropError> for VeltranoError {
    fn from(err: RustInteropError) -> Self {
        match err {
            RustInteropError::CargoError(msg) => VeltranoError::new(ErrorKind::InteropError, msg),
            RustInteropError::ParseError(msg) => VeltranoError::new(ErrorKind::ParseError, msg),
            RustInteropError::IoError(msg) => VeltranoError::new(ErrorKind::IoError, msg),
            RustInteropError::CrateNotFound(name) => VeltranoError::new(
                ErrorKind::CrateNotFound,
                format!("crate '{}' not found", name),
            ),
        }
    }
}

/// Convert TypeCheckError to VeltranoError
impl From<TypeCheckError> for VeltranoError {
    fn from(err: TypeCheckError) -> Self {
        use super::Span;

        match err {
            TypeCheckError::TypeMismatch {
                expected,
                actual,
                location,
            } => VeltranoError::new(
                ErrorKind::TypeMismatch,
                format!("Type mismatch: expected {:?}, found {:?}", expected, actual),
            )
            .with_span(Span::single(location)),
            TypeCheckError::_TypeMismatchWithSuggestion {
                expected,
                actual,
                location,
                suggestion,
            } => VeltranoError::new(
                ErrorKind::TypeMismatch,
                format!("Type mismatch: expected {:?}, found {:?}", expected, actual),
            )
            .with_span(Span::single(location))
            .with_help(suggestion),
            TypeCheckError::MethodNotFound {
                receiver_type,
                method,
                location,
            } => VeltranoError::new(
                ErrorKind::InvalidMethodCall,
                format!("Method '{}' not found for type {:?}", method, receiver_type),
            )
            .with_span(Span::single(location)),
            TypeCheckError::_MethodNotFoundWithSuggestion {
                receiver_type,
                method,
                location,
                suggestion,
            } => VeltranoError::new(
                ErrorKind::InvalidMethodCall,
                format!("Method '{}' not found for type {:?}", method, receiver_type),
            )
            .with_span(Span::single(location))
            .with_help(suggestion),
            TypeCheckError::FieldNotFound {
                object_type,
                field,
                location,
            } => VeltranoError::new(
                ErrorKind::TypeError,
                format!("Field '{}' not found for type {:?}", field, object_type),
            )
            .with_span(Span::single(location)),
            TypeCheckError::_FieldNotFoundWithSuggestion {
                object_type,
                field,
                location,
                suggestion,
            } => VeltranoError::new(
                ErrorKind::TypeError,
                format!("Field '{}' not found for type {:?}", field, object_type),
            )
            .with_span(Span::single(location))
            .with_help(suggestion),
            TypeCheckError::ArgumentCountMismatch {
                function,
                expected,
                actual,
                location,
            } => VeltranoError::new(
                ErrorKind::TypeError,
                format!(
                    "Function '{}' expects {} arguments, but {} were provided",
                    function, expected, actual
                ),
            )
            .with_span(Span::single(location)),
            TypeCheckError::VariableNotFound { name, location } => VeltranoError::new(
                ErrorKind::UndefinedVariable,
                format!("Variable '{}' not found", name),
            )
            .with_span(Span::single(location)),
            TypeCheckError::FunctionNotFound { name, location } => VeltranoError::new(
                ErrorKind::UndefinedFunction,
                format!("Function '{}' not found", name),
            )
            .with_span(Span::single(location)),
            TypeCheckError::AmbiguousMethodCall {
                method,
                receiver_type,
                candidates,
                location,
            } => VeltranoError::new(
                ErrorKind::AmbiguousType,
                format!(
                    "Ambiguous method call '{}' for type {:?}",
                    method, receiver_type
                ),
            )
            .with_span(Span::single(location))
            .with_note(format!("Candidates: {}", candidates.join(", "))),
            TypeCheckError::InvalidTypeConstructor { message, location } => {
                VeltranoError::new(ErrorKind::TypeError, message).with_span(Span::single(location))
            }
            TypeCheckError::UnsupportedFeature { feature, location } => {
                VeltranoError::new(ErrorKind::UnsupportedFeature, feature)
                    .with_span(Span::single(location))
            }
            _ => {
                // For any remaining error types, use Debug formatting
                VeltranoError::new(ErrorKind::TypeError, format!("{:?}", err))
            }
        }
    }
}
