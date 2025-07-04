pub mod error;
mod expressions;
mod imports;
mod method_calls;
mod statements;
mod types;

use crate::ast::*;
use crate::builtins::BuiltinRegistry;
use crate::error::VeltranoError;
use crate::rust_interop::RustInteropRegistry;
use crate::types::*;

pub use error::{MethodResolution, TypeCheckError};
use imports::{register_builtin_imports, ImportHandler};

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
        let mut trait_checker = RustInteropRegistry::new();
        let mut import_handler = ImportHandler::new();

        // Register built-in imports before any user code
        register_builtin_imports(&mut import_handler, &mut trait_checker);

        let mut checker = Self {
            env: TypeEnvironment::new(),
            trait_checker,
            builtin_registry: BuiltinRegistry::new(),
            import_handler,
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

        // Check for any import errors that were collected
        errors.extend(self.import_handler.get_import_errors().iter().cloned());

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Type check with VeltranoError for unified error handling
    pub fn check_program_unified(&mut self, program: &Program) -> Result<(), Vec<VeltranoError>> {
        self.check_program(program).map_err(|errors| {
            errors
                .into_iter()
                .map(|e| error::ErrorAnalyzer::enhance_error(e).into())
                .collect()
        })
    }
}
