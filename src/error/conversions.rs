//! Conversion utilities for migrating existing error types to VeltranoError

use super::{ErrorKind, VeltranoError};
use crate::codegen::CodegenError;
use crate::rust_interop::RustInteropError;
use crate::type_checker::TypeCheckError;

/// Convert CodegenError to VeltranoError
impl From<CodegenError> for VeltranoError {
    fn from(err: CodegenError) -> Self {
        match err {
            CodegenError::InvalidDataClassSyntax {
                constructor,
                reason,
            } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!(
                    "Invalid data class syntax for '{}': {}",
                    constructor, reason
                ),
            ),
            CodegenError::InvalidShorthandUsage {
                field_name,
                context,
            } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!("Invalid shorthand usage of '{}' in {}", field_name, context),
            ),
            CodegenError::InvalidBuiltinArguments { builtin, reason } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!("Invalid arguments for builtin '{}': {}", builtin, reason),
            ),
            CodegenError::MissingImport { method, type_name } => VeltranoError::new(
                ErrorKind::CodegenError,
                format!(
                    "Method '{}' requires import for type '{}'",
                    method, type_name
                ),
            ),
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
        // For now, just convert to a simple error message using Debug
        // In the future, we'll preserve source location information
        let message = format!("{:?}", err);
        match &err {
            TypeCheckError::TypeMismatch { .. }
            | TypeCheckError::TypeMismatchWithSuggestion { .. } => {
                VeltranoError::new(ErrorKind::TypeMismatch, message)
            }
            TypeCheckError::MethodNotFound { .. }
            | TypeCheckError::MethodNotFoundWithSuggestion { .. } => {
                VeltranoError::new(ErrorKind::InvalidMethodCall, message)
            }
            TypeCheckError::FieldNotFound { .. }
            | TypeCheckError::FieldNotFoundWithSuggestion { .. } => {
                VeltranoError::new(ErrorKind::TypeError, message)
            }
            TypeCheckError::ArgumentCountMismatch { .. } => {
                VeltranoError::new(ErrorKind::TypeError, message)
            }
            _ => VeltranoError::new(ErrorKind::TypeError, message),
        }
    }
}

/// Helper trait for converting String errors to VeltranoError
pub trait IntoVeltranoError {
    fn into_syntax_error(self) -> VeltranoError;
    fn into_parse_error(self) -> VeltranoError;
}

impl IntoVeltranoError for String {
    fn into_syntax_error(self) -> VeltranoError {
        VeltranoError::new(ErrorKind::SyntaxError, self)
    }

    fn into_parse_error(self) -> VeltranoError {
        VeltranoError::new(ErrorKind::ParseError, self)
    }
}
