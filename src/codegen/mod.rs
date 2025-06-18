//! Rust code generation from Veltrano AST.
//! 
//! Orchestrates the transpilation process and maintains generation state.

mod comments;
mod expressions;
mod formatting;
mod statements;
mod types;
mod utils;

use crate::ast::query::AstQuery;
use crate::ast::*;
use crate::ast_types::StmtExt;
use crate::config::Config;
use crate::error::{SourceLocation, VeltranoError};
use crate::rust_interop::RustInteropRegistry;
use crate::type_checker::MethodResolution;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Errors that can occur during code generation
#[derive(Debug)]
pub enum CodegenError {
    /// Invalid syntax when calling a data class constructor
    InvalidDataClassSyntax {
        constructor: String,
        reason: String,
        location: SourceLocation,
    },
    /// Shorthand syntax used in wrong context
    InvalidShorthandUsage {
        field_name: String,
        context: String,
        location: SourceLocation,
    },
    /// Invalid arguments for built-in functions
    InvalidBuiltinArguments {
        builtin: String,
        reason: String,
        location: SourceLocation,
    },
    /// Method requires import but wasn't imported
    MissingImport {
        method: String,
        type_name: String,
        location: SourceLocation,
    },
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::InvalidDataClassSyntax {
                constructor,
                reason,
                location: _,
            } => {
                write!(
                    f,
                    "Invalid syntax for data class '{}': {}",
                    constructor, reason
                )
            }
            CodegenError::InvalidShorthandUsage {
                field_name,
                context,
                location: _,
            } => {
                write!(
                    f,
                    "Shorthand syntax (.{}) is only valid for data class constructors, not {}",
                    field_name, context
                )
            }
            CodegenError::InvalidBuiltinArguments {
                builtin,
                reason,
                location: _,
            } => {
                write!(f, "Invalid arguments for {}: {}", builtin, reason)
            }
            CodegenError::MissingImport {
                method,
                type_name,
                location: _,
            } => {
                write!(f, "Method '{}' requires an explicit import. Add 'import {}.{}' at the top of your file.", method, type_name, method)
            }
        }
    }
}

impl std::error::Error for CodegenError {}

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    imports: HashMap<String, (String, String)>, // alias/method_name -> (type_name, method_name)
    local_functions: HashSet<String>,           // Set of locally defined function names
    local_functions_with_bump: HashSet<String>, // Functions that need bump parameter
    data_classes_with_lifetime: HashSet<String>, // Track data classes that need lifetime parameters
    data_classes: HashSet<String>,              // Track all data classes
    generating_bump_function: bool, // Track when generating function with bump parameter
    trait_checker: RustInteropRegistry, // For trait-based type checking
    config: Config,
    method_resolutions: HashMap<usize, MethodResolution>, // Method call ID -> resolved import
}

impl CodeGenerator {
    pub fn with_config(config: Config) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            imports: HashMap::new(),
            local_functions: HashSet::new(),
            local_functions_with_bump: HashSet::new(),
            data_classes_with_lifetime: HashSet::new(),
            data_classes: HashSet::new(),
            generating_bump_function: false,
            trait_checker: RustInteropRegistry::new(),
            config,
            method_resolutions: HashMap::new(),
        }
    }

    /// Set method resolutions from the type checker
    pub fn set_method_resolutions(&mut self, resolutions: HashMap<usize, MethodResolution>) {
        self.method_resolutions = resolutions;
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, VeltranoError> {
        // First pass: collect all locally defined function names and data classes with lifetimes
        for stmt in &program.statements {
            match stmt {
                Stmt::FunDecl(fun_decl) => {
                    self.local_functions.insert(fun_decl.name.clone());
                    if fun_decl.has_hidden_bump {
                        self.local_functions_with_bump.insert(fun_decl.name.clone());
                    }
                }
                Stmt::DataClass(data_class) => {
                    // Track all data classes
                    self.data_classes.insert(data_class.name.clone());

                    // Check if this data class needs lifetime parameters
                    let needs_lifetime = data_class
                        .fields
                        .iter()
                        .any(|field| self.type_needs_lifetime(&field.field_type.node));
                    if needs_lifetime {
                        self.data_classes_with_lifetime
                            .insert(data_class.name.clone());
                    }
                }
                _ => {}
            }
        }

        // Skip bumpalo import - use fully qualified names instead

        // Second pass: generate code
        for stmt in &program.statements {
            self.generate_statement(stmt)?;
        }
        Ok(self.output.clone())
    }








    fn check_function_needs_bump(&self, fun_decl: &FunDeclStmt) -> bool {
        // First check for direct bump allocation usage
        if AstQuery::function_requires_bump(fun_decl) {
            return true;
        }

        // Also check if this function calls other functions that use bump
        let mut uses_bump = false;
        let _ = fun_decl.body.walk_expressions(&mut |expr| {
            if let Expr::Call(call) = &expr.node {
                if let Expr::Identifier(name) = &call.callee.node {
                    if self.local_functions_with_bump.contains(name) {
                        uses_bump = true;
                        return Err(()); // Early exit
                    }
                }
            }
            Ok::<(), ()>(())
        });

        uses_bump
    }
}
