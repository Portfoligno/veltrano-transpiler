//! Statement code generation for the Veltrano language
//!
//! This module handles generation of all statement types including:
//! - Variable declarations
//! - Function declarations
//! - Control flow (if, while)
//! - Data classes
//! - Imports

use crate::ast::*;
use crate::ast_types::CommentContext;
use crate::comments::{Comment, CommentStyle};
use crate::error::VeltranoError;
use crate::rust_interop::camel_to_snake_case;
use super::CodeGenerator;

impl CodeGenerator {
    /// Generate code for any statement type
    pub(super) fn generate_statement(&mut self, stmt: &Stmt) -> Result<(), VeltranoError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.indent();

                // Check if this is a method call with its own comment
                let method_comment = if let Expr::MethodCall(method_call) = &expr.node {
                    method_call.inline_comment.clone()
                } else {
                    None
                };

                self.generate_expression(expr)?;
                self.output.push(';');

                // Generate method comment after semicolon if present
                if let Some(comment) = method_comment {
                    self.generate_inline_comment(&Some(comment));
                }

                self.output.push('\n');
            }
            Stmt::VarDecl(var_decl) => {
                self.generate_var_declaration(var_decl)?;
            }
            Stmt::FunDecl(fun_decl) => {
                self.generate_function_declaration(fun_decl)?;
            }
            Stmt::If(if_stmt) => {
                self.generate_if_statement(if_stmt)?;
            }
            Stmt::While(while_stmt) => {
                self.generate_while_statement(while_stmt)?;
            }
            Stmt::Return(expr) => {
                self.indent();
                self.output.push_str("return");
                if let Some(expr) = expr {
                    self.output.push(' ');
                    self.generate_expression(expr)?;
                }
                self.output.push(';');
                self.output.push('\n');
            }
            Stmt::Block(statements) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Stmt::Comment(comment) => {
                if self.config.preserve_comments {
                    self.generate_comment(comment);
                }
            }
            Stmt::Import(import) => {
                // Track the import for later use
                let key = import
                    .alias
                    .clone()
                    .unwrap_or_else(|| import.method_name.clone());
                self.imports
                    .insert(key, (import.type_name.clone(), import.method_name.clone()));
                // Don't generate any Rust code for imports
            }
            Stmt::DataClass(data_class) => {
                self.generate_data_class(data_class);
            }
        }
        Ok(())
    }

    /// Generate code for variable declarations
    fn generate_var_declaration(&mut self, var_decl: &VarDeclStmt) -> Result<(), VeltranoError> {
        self.indent();

        self.output.push_str("let ");

        let snake_name = camel_to_snake_case(&var_decl.name);
        self.output.push_str(&snake_name);

        if let Some(type_annotation) = &var_decl.type_annotation {
            self.output.push_str(": ");
            self.generate_type(&type_annotation.node);
        }

        if let Some(initializer) = &var_decl.initializer {
            self.output.push_str(" = ");

            // Collect all comments from method chain if this is a method call
            let method_chain_comments = self.collect_method_chain_comments(initializer);

            self.generate_expression(initializer)?;
            self.output.push(';');

            // Generate all method chain comments after semicolon
            if !method_chain_comments.is_empty() {
                // Output each comment in its original style
                for (i, comment) in method_chain_comments.iter().enumerate() {
                    if i == 0 {
                        // First comment uses its original whitespace
                        self.generate_inline_comment(&Some(comment.clone()));
                    } else {
                        // Subsequent comments get minimal whitespace to stay on same line
                        let (content, _) = comment;
                        self.generate_inline_comment(&Some((content.clone(), " ".to_string())));
                    }
                }
            }
        } else {
            self.output.push(';');
        }

        self.output.push('\n');
        Ok(())
    }

    /// Generate code for function declarations
    pub(super) fn generate_function_declaration(
        &mut self,
        fun_decl: &FunDeclStmt,
    ) -> Result<(), VeltranoError> {
        self.indent();
        self.output.push_str("fn ");
        let snake_name = camel_to_snake_case(&fun_decl.name);
        self.output.push_str(&snake_name);

        // Add lifetime parameter if this function has a hidden bump parameter
        if fun_decl.has_hidden_bump {
            self.output.push_str("<'a>");
            self.generating_bump_function = true;
        }

        self.output.push('(');

        // Check if we should use multiline formatting for parameters
        let use_multiline = fun_decl.params.iter().any(|p| p.inline_comment.is_some());

        if use_multiline && !fun_decl.params.is_empty() {
            self.generate_multiline_params(&fun_decl.params, fun_decl.has_hidden_bump);
        } else {
            self.generate_comma_separated_params(&fun_decl.params, fun_decl.has_hidden_bump);
        }

        self.output.push(')');

        if let Some(return_type) = &fun_decl.return_type {
            self.output.push_str(" -> ");
            self.generate_type(&return_type.node);
        }

        self.output.push(' ');

        // Special handling for main function: only initialize bump allocator if needed
        if fun_decl.name == "main" {
            self.output.push_str("{\n");
            self.indent_level += 1;

            // Check if bump allocation is actually used in the main function body
            let needs_bump = self.check_function_needs_bump(fun_decl);
            if needs_bump {
                self.indent();
                self.output.push_str("let bump = &bumpalo::Bump::new();\n");
            }

            // Generate the body content but skip the outer braces since we're handling them
            if let Stmt::Block(statements) = fun_decl.body.as_ref() {
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
            } else {
                self.generate_statement(&fun_decl.body)?;
            }

            self.indent_level -= 1;
            self.indent();
            self.output.push_str("}\n");
        } else {
            self.generate_statement(&fun_decl.body)?;
        }

        // Reset bump function flag
        self.generating_bump_function = false;
        Ok(())
    }

    /// Generate code for if statements
    fn generate_if_statement(&mut self, if_stmt: &IfStmt) -> Result<(), VeltranoError> {
        self.indent();
        self.output.push_str("if ");
        self.generate_expression(&if_stmt.condition)?;
        self.output.push(' ');

        self.generate_statement(&if_stmt.then_branch)?;

        if let Some(else_branch) = &if_stmt.else_branch {
            self.indent();
            self.output.push_str("else ");
            self.generate_statement(else_branch)?;
        }
        Ok(())
    }

    /// Generate code for while statements
    fn generate_while_statement(&mut self, while_stmt: &WhileStmt) -> Result<(), VeltranoError> {
        self.indent();

        // Check if this is an infinite loop (while true)
        if let Expr::Literal(LiteralExpr::Bool(true)) = &while_stmt.condition.node {
            self.output.push_str("loop ");
        } else {
            self.output.push_str("while ");
            self.generate_expression(&while_stmt.condition)?;
            self.output.push(' ');
        }

        self.generate_statement(&while_stmt.body)?;
        Ok(())
    }

    /// Generate code for data class declarations
    pub(super) fn generate_data_class(&mut self, data_class: &DataClassStmt) {
        // Check if any fields are reference types
        let needs_lifetime = data_class
            .fields
            .iter()
            .any(|field| self.type_needs_lifetime(&field.field_type.node));

        self.indent();
        self.output.push_str("#[derive(Debug, Clone)]\n");
        self.indent();
        self.output.push_str("pub struct ");
        self.output.push_str(&data_class.name);

        if needs_lifetime {
            self.output.push_str("<'a>");
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;

        // Generate fields
        for field in &data_class.fields {
            self.indent();
            self.output.push_str("pub ");
            self.output.push_str(&camel_to_snake_case(&field.name));
            self.output.push_str(": ");

            // Generate the field type with lifetime if needed
            if needs_lifetime {
                self.generate_data_class_field_type(&field.field_type.node);
            } else {
                self.generate_type(&field.field_type.node);
            }

            // Always add comma for Rust struct fields
            self.output.push(',');

            // Generate inline comment if present
            self.generate_inline_comment(&field.inline_comment);
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}\n\n");
    }
}
