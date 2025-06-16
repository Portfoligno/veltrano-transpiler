//! Statement checking logic for the type checker
//!
//! This module contains logic for checking various types of statements
//! including variable declarations, function declarations, control flow,
//! and data class declarations.

use crate::ast::query::AstQuery;
use crate::ast::*;
use crate::error::SourceLocation;
use crate::types::{DataClassDefinition, DataClassFieldSignature, FunctionSignature, VeltranoType};

use super::error::TypeCheckError;
use super::types::TypeValidator;
use super::VeltranoTypeChecker;

impl VeltranoTypeChecker {
    /// Check a statement for type correctness
    pub(super) fn check_statement(&mut self, stmt: &Stmt) -> Result<(), TypeCheckError> {
        match stmt {
            Stmt::VarDecl(var_decl) => self.check_var_declaration(var_decl),
            Stmt::FunDecl(fun_decl) => self.check_function_declaration(fun_decl),
            Stmt::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Stmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.check_expression(expr)?;
                }
                Ok(())
            }
            Stmt::If(if_stmt) => {
                self.check_expression(&if_stmt.condition)?;
                self.check_statement(&if_stmt.then_branch)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.check_statement(else_branch)?;
                }
                Ok(())
            }
            Stmt::While(while_stmt) => {
                self.check_expression(&while_stmt.condition)?;
                self.check_statement(&while_stmt.body)?;
                Ok(())
            }
            Stmt::Block(statements) => {
                self.env.enter_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.env.exit_scope();
                Ok(())
            }
            Stmt::DataClass(data_class) => self.check_data_class_declaration(data_class),
            Stmt::Import(import) => self.check_import_statement(import),
            Stmt::Comment(_) => {
                // Comments don't need type checking
                Ok(())
            }
        }
    }

    /// Check import statement and register it for method resolution
    pub(super) fn check_import_statement(
        &mut self,
        import: &ImportStmt,
    ) -> Result<(), TypeCheckError> {
        self.import_handler
            .check_import_statement(import, &mut self.trait_checker)
    }

    /// Check data class declaration and register it in the environment
    pub(super) fn check_data_class_declaration(
        &mut self,
        data_class: &DataClassStmt,
    ) -> Result<(), TypeCheckError> {
        // Validate all field types
        for field in &data_class.fields {
            self.validate_type(&field.field_type.node, field.field_type.span.start.clone())?;
        }

        // Create data class definition
        let fields: Vec<DataClassFieldSignature> = data_class
            .fields
            .iter()
            .map(|f| DataClassFieldSignature {
                name: f.name.clone(),
                field_type: f.field_type.node.clone(),
            })
            .collect();

        let definition = DataClassDefinition {
            _name: data_class.name.clone(),
            fields,
        };

        // Register the data class in the environment
        self.env
            .declare_data_class(data_class.name.clone(), definition);

        Ok(())
    }

    /// Check variable declaration
    pub(super) fn check_var_declaration(
        &mut self,
        var_decl: &VarDeclStmt,
    ) -> Result<(), TypeCheckError> {
        // Validate type annotation if present
        if let Some(declared_type) = &var_decl.type_annotation {
            self.validate_type(&declared_type.node, declared_type.span.start.clone())?;
        }

        if let Some(initializer) = &var_decl.initializer {
            // Pass expected type for inference if available
            let init_type = if let Some(declared_type) = &var_decl.type_annotation {
                self.check_expression_with_expected_type(initializer, Some(&declared_type.node))?
            } else {
                self.check_expression(initializer)?
            };

            if let Some(declared_type) = &var_decl.type_annotation {
                let expected_type = declared_type.node.clone();

                // Strict type checking: types must match exactly
                if !self.types_equal(&expected_type, &init_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_type,
                        actual: init_type,
                        location: initializer.span.start.clone(),
                    });
                }
            }

            // Declare the variable in the environment
            self.env.declare_variable(var_decl.name.clone(), init_type);
        }

        Ok(())
    }

    /// Recursively collect function signatures from a statement (including nested functions)
    pub(super) fn collect_function_signatures_from_statement(
        &mut self,
        stmt: &Stmt,
    ) -> Result<(), TypeCheckError> {
        // Use the query infrastructure to find all function declarations
        let function_decls = AstQuery::find_function_decls(stmt);

        // Collect signatures for all found functions
        for fun_decl in function_decls {
            self.collect_function_signature(fun_decl)?;
        }

        Ok(())
    }

    /// Collect function signature in the first pass (doesn't check body)
    pub(super) fn collect_function_signature(
        &mut self,
        fun_decl: &FunDeclStmt,
    ) -> Result<(), TypeCheckError> {
        // Validate parameter types
        for param in &fun_decl.params {
            self.validate_type(&param.param_type.node, param.param_type.span.start.clone())?;
        }

        // Validate return type if present
        if let Some(return_type) = &fun_decl.return_type {
            self.validate_type(&return_type.node, return_type.span.start.clone())?;
        }

        // Create function signature and add to environment
        let param_types: Vec<VeltranoType> = fun_decl
            .params
            .iter()
            .map(|p| p.param_type.node.clone())
            .collect();

        let return_type = fun_decl
            .return_type
            .as_ref()
            .map(|t| t.node.clone())
            .unwrap_or_else(|| VeltranoType::unit());

        let signature = FunctionSignature {
            name: fun_decl.name.clone(),
            parameters: param_types,
            return_type,
        };

        self.env.declare_function(fun_decl.name.clone(), signature);

        Ok(())
    }

    /// Check function declaration
    pub(super) fn check_function_declaration(
        &mut self,
        fun_decl: &FunDeclStmt,
    ) -> Result<(), TypeCheckError> {
        // Function signature already collected in first pass, just check the body

        // Check function body
        self.env.enter_scope();

        // Add parameters to scope
        for param in &fun_decl.params {
            self.env
                .declare_variable(param.name.clone(), param.param_type.node.clone());
        }

        // First collect any nested function signatures within the body
        self.collect_function_signatures_from_statement(&fun_decl.body)?;

        // Then check the body
        self.check_statement(&fun_decl.body)?;

        self.env.exit_scope();

        Ok(())
    }

    /// Validate a type recursively, checking for invalid type constructor usage
    pub(super) fn validate_type(
        &mut self,
        veltrano_type: &VeltranoType,
        location: SourceLocation,
    ) -> Result<(), TypeCheckError> {
        TypeValidator::validate_type(veltrano_type, &mut self.trait_checker, location)
    }

    /// Core type equality check - no implicit conversion logic
    pub(super) fn types_equal(&self, a: &VeltranoType, b: &VeltranoType) -> bool {
        TypeValidator::types_equal(a, b)
    }
}
