//! Rust code generation from Veltrano AST
//!
//! This module is responsible for converting the Veltrano AST into Rust source code.
//! It handles all aspects of code generation including expression evaluation,
//! statement generation, type conversions, and proper formatting.
//!
//! The code generator is organized into sub-modules:
//! - `expressions` - Expression generation
//! - `statements` - Statement generation
//! - `types` - Type generation with ownership
//! - `formatting` - Code formatting utilities
//! - `comments` - Comment generation

mod comments;
mod expressions;
mod formatting;
mod statements;
mod types;
mod utils;

use crate::ast::query::AstQuery;
use crate::ast::*;
use crate::ast_types::StmtExt;
use crate::comments::{Comment, CommentStyle};
use crate::config::Config;
use crate::error::{SourceLocation, Span, VeltranoError};
use crate::rust_interop::camel_to_snake_case;
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

    fn generate_statement(&mut self, stmt: &Stmt) -> Result<(), VeltranoError> {
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

    fn generate_function_declaration(
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

    fn generate_data_class(&mut self, data_class: &DataClassStmt) {
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



    fn generate_comma_separated_params(&mut self, params: &[Parameter], include_bump: bool) {
        let mut first = true;

        if include_bump {
            self.output.push_str("bump: &'a bumpalo::Bump");
            first = false;
        }

        for param in params {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            let snake_name = camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type.node);
            self.generate_inline_comment_as_block(&param.inline_comment);
        }
    }

    fn generate_multiline_params(&mut self, params: &[Parameter], include_bump: bool) {
        self.output.push('\n');
        self.indent_level += 1;

        if include_bump {
            self.indent();
            self.output.push_str("bump: &'a bumpalo::Bump");
            if !params.is_empty() {
                self.output.push(',');
            }
            self.output.push('\n');
        }

        for (i, param) in params.iter().enumerate() {
            self.indent();
            let snake_name = camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type.node);

            // Add comma if not the last parameter
            if i < params.len() - 1 {
                self.output.push(',');
            }

            // Generate inline comment as line comment, not block comment
            self.generate_inline_comment(&param.inline_comment);
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.indent();
    }

    fn generate_comma_separated_args_for_struct_init(
        &mut self,
        args: &[Argument],
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        let mut first = true;
        for arg in args {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            match arg {
                Argument::Bare(_, _) => {
                    // We need the data class name for the error, but we don't have it here
                    // For now, use a generic message
                    return Err(CodegenError::InvalidDataClassSyntax {
                        constructor: "data class".to_string(),
                        reason: "Data class constructors don't support positional arguments. Use named arguments or .field shorthand syntax".to_string(),
                        location: call_span.start.clone(),
                    }.into());
                }
                Argument::Named(name, expr, comment) => {
                    self.output.push_str(&camel_to_snake_case(name));
                    self.output.push_str(": ");
                    self.generate_expression(expr)?;
                    self.generate_inline_comment(comment);
                }
                Argument::Shorthand(field_name, comment) => {
                    // Shorthand: generate field_name (variable matches field name)
                    self.output.push_str(&camel_to_snake_case(field_name));
                    self.generate_inline_comment(comment);
                }
                Argument::StandaloneComment(_, _) => {
                    // For single-line struct initialization, ignore standalone comments
                    first = true; // Don't add comma before next real argument
                }
            }
        }
        Ok(())
    }

    fn generate_comma_separated_args_for_function_call_with_multiline(
        &mut self,
        args: &[Argument],
        is_multiline: bool,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        if is_multiline && !args.is_empty() {
            // Generate multiline format
            self.output.push('\n');
            for (i, arg) in args.iter().enumerate() {
                self.indent_level += 1;
                self.indent();

                match arg {
                    Argument::Bare(expr, comment) => {
                        self.generate_expression(expr)?;
                        if i < args.len() - 1 {
                            self.output.push(',');
                        }
                        self.generate_inline_comment(comment);
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        self.generate_expression(expr)?;
                        if i < args.len() - 1 {
                            self.output.push(',');
                        }
                        self.generate_inline_comment(comment);
                    }
                    Argument::Shorthand(field_name, _) => {
                        return Err(CodegenError::InvalidShorthandUsage {
                            field_name: field_name.clone(),
                            context: "function calls".to_string(),
                            location: call_span.start.clone(),
                        }
                        .into());
                    }
                    Argument::StandaloneComment(content, whitespace) => {
                        // Generate standalone comment as its own line
                        if self.config.preserve_comments {
                            // Following the pattern from generate_comment for regular statements:
                            // The loop already called indent() to add base indentation.
                            // Now add any extra whitespace preserved by the lexer.
                            let comment =
                                Comment::from_tuple((content.clone(), whitespace.clone()));
                            self.output.push_str(&comment.whitespace);

                            match comment.style {
                                CommentStyle::Block => {
                                    self.output.push_str(&comment.content);
                                }
                                CommentStyle::Line => {
                                    self.output.push_str("//");
                                    self.output.push_str(&comment.content);
                                }
                            }
                        }
                        // Note: No comma or expression - this is just a comment line
                    }
                }
                self.output.push('\n');
                self.indent_level -= 1;
            }
            self.indent();
        } else {
            // Generate single line format
            let mut first = true;
            for arg in args {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                match arg {
                    Argument::Bare(expr, comment) => {
                        self.generate_expression(expr)?;
                        self.generate_inline_comment_as_block(comment);
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        self.generate_expression(expr)?;
                        self.generate_inline_comment_as_block(comment);
                    }
                    Argument::Shorthand(field_name, _) => {
                        return Err(CodegenError::InvalidShorthandUsage {
                            field_name: field_name.clone(),
                            context: "function calls".to_string(),
                            location: call_span.start.clone(),
                        }
                        .into());
                    }
                    Argument::StandaloneComment(_, _) => {
                        // For single-line calls, standalone comments force multiline format
                        // This should not happen in practice as standalone comments should trigger multiline
                        // But handle it gracefully by ignoring the comment in single-line format
                        first = true; // Don't add comma before next real argument
                    }
                }
            }
        }
        Ok(())
    }


    fn generate_comment(&mut self, comment: &CommentStmt) {
        match comment.context {
            CommentContext::OwnLine => {
                // Own-line comments get indentation
                self.indent();
            }
            CommentContext::EndOfLine => {
                // EndOfLine comments: remove the trailing newline from previous statement
                if self.output.ends_with('\n') {
                    self.output.pop();
                }
            }
        }

        // Always apply the preserved whitespace
        self.output.push_str(&comment.preceding_whitespace);

        if comment.is_block_comment {
            self.output.push_str("/*");
            self.output.push_str(&comment.content);
            self.output.push_str("*/");
        } else {
            self.output.push_str("//");
            self.output.push_str(&comment.content);
        }

        // Always add newline at the end
        self.output.push('\n');
    }

    fn generate_inline_comment(&mut self, inline_comment: &Option<(String, String)>) {
        if let Some((content, whitespace)) = inline_comment {
            if self.config.preserve_comments {
                let comment = Comment::from_tuple((content.clone(), whitespace.clone()));
                self.output.push_str(&comment.whitespace);

                // Use Comment to determine style and format appropriately
                match comment.style {
                    CommentStyle::Block => {
                        // Block comment - output as-is
                        self.output.push_str(&comment.content);
                    }
                    CommentStyle::Line => {
                        // Line comment - add // prefix
                        self.output.push_str("//");
                        self.output.push_str(&comment.content);
                    }
                }
            }
        }
    }

    fn generate_inline_comment_as_block(&mut self, inline_comment: &Option<(String, String)>) {
        if let Some((content, whitespace)) = inline_comment {
            if self.config.preserve_comments {
                let comment = Comment::from_tuple((content.clone(), whitespace.clone()));
                self.output.push_str(&comment.whitespace);

                // Use Comment's to_block_style method to convert if needed
                let block_comment = comment.to_block_style();
                self.output.push_str(&block_comment.content);
            }
        }
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
