//! Expression code generation for the Veltrano language
//!
//! This module handles generation of all expression types including:
//! - Literals (int, string, bool, null, unit)
//! - Binary and unary operations
//! - Function and method calls
//! - Field access

use crate::ast::*;
use crate::ast_types::Argument;
use crate::comments::{Comment, CommentStyle};
use crate::error::{VeltranoError, Span};
use crate::rust_interop::camel_to_snake_case;
use super::{CodeGenerator, CodegenError};

impl CodeGenerator {
    /// Generate code for any expression type
    pub(super) fn generate_expression(&mut self, expr: &LocatedExpr) -> Result<(), VeltranoError> {
        match &expr.node {
            Expr::Literal(literal) => {
                self.generate_literal(literal, expr.span.clone());
            }
            Expr::Identifier(name) => {
                let snake_name = camel_to_snake_case(name);
                self.output.push_str(&snake_name);
            }
            Expr::Unary(unary) => {
                match &unary.operator {
                    UnaryOp::Minus => {
                        self.output.push('-');
                        // Wrap non-simple expressions in parentheses
                        match &unary.operand.node {
                            Expr::Literal(_) | Expr::Identifier(_) => {
                                self.generate_expression(&unary.operand)?;
                            }
                            Expr::Unary(_) => {
                                // Wrap nested unary to avoid -- (double negation)
                                self.output.push('(');
                                self.generate_expression(&unary.operand)?;
                                self.output.push(')');
                            }
                            _ => {
                                self.output.push('(');
                                self.generate_expression(&unary.operand)?;
                                self.output.push(')');
                            }
                        }
                    }
                }
            }
            Expr::Binary(binary) => {
                self.generate_expression(&binary.left)?;
                
                // Generate comment after left operand if present
                if let Some((content, whitespace)) = &binary.comment_after_left {
                    if self.config.preserve_comments {
                        let comment = Comment::from_tuple((content.clone(), whitespace.clone()));
                        
                        // Use Comment to determine style and format appropriately
                        match comment.style {
                            CommentStyle::Block => {
                                // Block comment - can stay inline
                                self.output.push(' ');
                                self.output.push_str(&comment.whitespace);
                                self.output.push_str(&comment.content);
                                self.output.push(' ');
                            }
                            CommentStyle::Line => {
                                // Line comment - needs to be on its own line
                                self.output.push_str("  ");
                                self.output.push_str("//");
                                self.output.push_str(&comment.content);
                                self.output.push('\n');
                                // Add indentation for the next line
                                for _ in 0..self.indent_level {
                                    self.output.push_str("    ");
                                }
                            }
                        }
                    } else {
                        self.output.push(' ');
                    }
                } else {
                    self.output.push(' ');
                }
                
                self.generate_binary_operator(&binary.operator);
                
                // Generate comment after operator if present
                if let Some((content, whitespace)) = &binary.comment_after_operator {
                    if self.config.preserve_comments {
                        let comment = Comment::from_tuple((content.clone(), whitespace.clone()));
                        
                        // Use Comment to determine style and format appropriately
                        match comment.style {
                            CommentStyle::Block => {
                                // Block comment - can stay inline
                                self.output.push(' ');
                                self.output.push_str(&comment.whitespace);
                                self.output.push_str(&comment.content);
                                self.output.push(' ');
                            }
                            CommentStyle::Line => {
                                // Line comment - needs to be on its own line
                                self.output.push_str("  ");
                                self.output.push_str("//");
                                self.output.push_str(&comment.content);
                                self.output.push('\n');
                                // Add indentation for the next line
                                for _ in 0..self.indent_level {
                                    self.output.push_str("    ");
                                }
                            }
                        }
                    } else {
                        self.output.push(' ');
                    }
                } else {
                    self.output.push(' ');
                }
                
                self.generate_expression(&binary.right)?;
            }
            Expr::Call(call) => self.generate_call_expression(call, expr.span.clone())?,
            Expr::MethodCall(method_call) => {
                self.generate_method_call_expression(method_call, expr.span.clone())?
            }
            Expr::FieldAccess(field_access) => {
                self.generate_field_access(field_access)?;
            }
        }
        Ok(())
    }

    /// Generate code for literal expressions
    fn generate_literal(&mut self, literal: &LiteralExpr, _span: Span) {
        match literal {
            LiteralExpr::Int(value) => {
                self.output.push_str(&value.to_string());
            }
            LiteralExpr::String(value) => {
                self.output.push('"');
                self.output.push_str(value);
                self.output.push('"');
            }
            LiteralExpr::Bool(value) => {
                self.output.push_str(&value.to_string());
            }
            LiteralExpr::Unit => {
                self.output.push_str("()");
            }
            LiteralExpr::Null => {
                self.output.push_str("None");
            }
        }
    }

    /// Generate code for binary operators
    fn generate_binary_operator(&mut self, op: &BinaryOp) {
        let op_str = match op {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        };
        self.output.push_str(op_str);
    }

    /// Generate code for generic function calls
    pub(super) fn generate_generic_call(
        &mut self,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        self.generate_expression(&call.callee)?;
        self.output.push('(');
        self.generate_comma_separated_args_for_function_call_with_multiline(
            &call.args,
            call.is_multiline,
            call_span,
        )?;
        self.output.push(')');
        Ok(())
    }

    /// Generate code for field access expressions
    fn generate_field_access(
        &mut self,
        field_access: &FieldAccessExpr,
    ) -> Result<(), VeltranoError> {
        self.generate_expression(&field_access.object)?;
        self.output.push('.');
        self.output
            .push_str(&camel_to_snake_case(&field_access.field));
        Ok(())
    }

    /// Generate code for function call expressions
    pub(super) fn generate_call_expression(
        &mut self,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        // First check if we have a type-checked resolution for this call (e.g., import alias)
        if let Some(resolution) = self.method_resolutions.get(&call.id) {
            // This is a resolved import alias (e.g., newVec -> Vec::new)
            let snake_method = camel_to_snake_case(&resolution.method_name);
            let type_name = resolution.rust_type.to_rust_syntax();

            self.output.push_str(&type_name);
            self.output.push_str("::");
            self.output.push_str(&snake_method);
            self.output.push('(');

            // Generate arguments
            self.generate_comma_separated_args_for_function_call_with_multiline(
                &call.args,
                call.is_multiline,
                call_span,
            )?;

            self.output.push(')');
            return Ok(());
        }

        if let Expr::Identifier(name) = &call.callee.node {
            // Check if this is a data class constructor
            if self.data_classes.contains(name) {
                // This is struct initialization (works with positional, named, or mixed arguments)
                self.output.push_str(name);

                if call.is_multiline {
                    // Multiline struct initialization
                    self.output.push_str(" {\n");
                    self.indent_level += 1;

                    for arg in call.args.iter() {
                        match arg {
                            Argument::StandaloneComment(content, whitespace) => {
                                // Generate standalone comment
                                self.indent();
                                if self.config.preserve_comments {
                                    let comment = Comment::new(
                                        content.clone(),
                                        whitespace.clone(),
                                        CommentStyle::Line,
                                    );
                                    self.output.push_str(&comment.whitespace);
                                    self.output.push_str("//");
                                    self.output.push_str(&comment.content);
                                }
                                self.output.push('\n');
                            }
                            Argument::Named(name, expr, comment) => {
                                self.indent();
                                self.output.push_str(&camel_to_snake_case(name));
                                self.output.push_str(": ");
                                self.generate_expression(expr)?;

                                // Always add comma for multiline struct fields
                                self.output.push(',');

                                self.generate_inline_comment(comment);
                                self.output.push('\n');
                            }
                            Argument::Shorthand(field_name, comment) => {
                                self.indent();
                                self.output.push_str(&camel_to_snake_case(field_name));

                                // Always add comma for multiline struct fields
                                self.output.push(',');

                                self.generate_inline_comment(comment);
                                self.output.push('\n');
                            }
                            Argument::Bare(_, _) => {
                                return Err(CodegenError::InvalidDataClassSyntax {
                                    constructor: name.clone(),
                                    reason: "Data class constructors don't support positional arguments. Use named arguments or .field shorthand syntax".to_string(),
                                    location: call_span.start.clone(),
                                }.into());
                            }
                        }
                    }

                    self.indent_level -= 1;
                    self.indent();
                    self.output.push('}');
                } else {
                    // Single-line struct initialization
                    self.output.push_str(" { ");
                    self.generate_comma_separated_args_for_struct_init(&call.args, call_span)?;
                    self.output.push_str(" }");
                }

                return Ok(());
            }

            if name == "MutRef" {
                // Special case: MutRef(value) becomes &mut (&value).clone()
                if call.args.len() != 1 {
                    return Err(CodegenError::InvalidBuiltinArguments {
                        builtin: "MutRef".to_string(),
                        reason: format!("requires exactly one argument, found {}", call.args.len()),
                        location: call_span.start.clone(),
                    }
                    .into());
                }
                self.output.push_str("&mut (&");
                match &call.args[0] {
                    Argument::Bare(expr, _) => {
                        self.generate_expression(expr)?;
                    }
                    Argument::Shorthand(field_name, _) => {
                        // Shorthand behaves like Bare - just generate the identifier
                        let snake_name = camel_to_snake_case(field_name);
                        self.output.push_str(&snake_name);
                    }
                    Argument::Named(_, _, _) => {
                        return Err(CodegenError::InvalidBuiltinArguments {
                            builtin: "MutRef".to_string(),
                            reason: "does not support named arguments".to_string(),
                            location: call_span.start.clone(),
                        }
                        .into());
                    }
                    Argument::StandaloneComment(_, _) => {
                        return Err(CodegenError::InvalidBuiltinArguments {
                            builtin: "MutRef".to_string(),
                            reason: "cannot have standalone comments as arguments".to_string(),
                            location: call_span.start.clone(),
                        }
                        .into());
                    }
                }
                self.output.push_str(").clone()");
            } else if self.local_functions.contains(name) {
                // Locally defined function: regular call with snake_case conversion
                let snake_name = camel_to_snake_case(name);
                self.output.push_str(&snake_name);
                self.output.push('(');

                // If this function has hidden bump, add bump as first argument
                if self.local_functions_with_bump.contains(name) {
                    self.output.push_str("bump");
                    if !call.args.is_empty() {
                        self.output.push_str(", ");
                    }
                }

                self.generate_comma_separated_args_for_function_call_with_multiline(
                    &call.args,
                    call.is_multiline,
                    call_span,
                )?;
                self.output.push(')');
            } else if let Some((type_name, original_method)) = self.imports.get(name) {
                // Imported function/constructor: use UFCS
                let snake_method = camel_to_snake_case(original_method);
                self.output.push_str(type_name);
                self.output.push_str("::");
                self.output.push_str(&snake_method);
                self.output.push('(');
                self.generate_comma_separated_args_for_function_call_with_multiline(
                    &call.args,
                    call.is_multiline,
                    call_span,
                )?;
                self.output.push(')');
            } else if self.is_rust_macro(name) {
                self.output.push_str(name);
                self.output.push('!');
                self.output.push('(');
                self.generate_comma_separated_args_for_function_call_with_multiline(
                    &call.args,
                    call.is_multiline,
                    call_span,
                )?;
                self.output.push(')');
            } else {
                // Default case for identifiers that aren't special
                self.generate_generic_call(call, call_span)?;
            }
        } else {
            self.generate_generic_call(call, call_span)?;
        }
        Ok(())
    }

    /// Generate code for method call expressions
    pub(super) fn generate_method_call_expression(
        &mut self,
        method_call: &MethodCallExpr,
        expr_span: Span,
    ) -> Result<(), VeltranoError> {
        // First check if we have a type-checked resolution for this method call
        crate::debug_println!(
            "DEBUG codegen: Looking for resolution for method call ID {}, method: {}",
            method_call.id,
            method_call.method
        );
        if let Some(resolution) = self.method_resolutions.get(&method_call.id) {
            // Use the resolved import
            crate::debug_println!(
                "DEBUG codegen: Found resolution - type: {:?}, method: {}",
                resolution.rust_type,
                resolution.method_name
            );
            let snake_method = camel_to_snake_case(&resolution.method_name);
            let type_name = resolution.rust_type.to_rust_syntax();

            self.output.push_str(&type_name);
            self.output.push_str("::");
            self.output.push_str(&snake_method);
            self.output.push('(');

            // First argument is the object
            self.generate_expression(&method_call.object)?;

            // Then the rest of the arguments
            for arg in &method_call.args {
                self.output.push_str(", ");
                self.generate_expression(arg)?;
            }
            self.output.push(')');
        } else if let Some((type_name, original_method)) = self.imports.get(&method_call.method) {
            // Fallback to old behavior for non-type-checked code
            let snake_method = camel_to_snake_case(original_method);
            self.output.push_str(type_name);
            self.output.push_str("::");
            self.output.push_str(&snake_method);
            self.output.push('(');

            // First argument is the object
            self.generate_expression(&method_call.object)?;

            // Then the rest of the arguments
            for arg in &method_call.args {
                self.output.push_str(", ");
                self.generate_expression(arg)?;
            }
            self.output.push(')');
        } else if method_call.method == "ref" && method_call.args.is_empty() {
            // Special case: ownedValue.ref() becomes &ownedValue
            // This converts Own<T> to T (which is &T in Rust)
            self.output.push('&');
            self.generate_expression(&method_call.object)?;
        } else if method_call.method == "bumpRef" && method_call.args.is_empty() {
            // Special case: value.bumpRef() becomes bump.alloc(value)
            // This moves the value to bump allocation
            self.output.push_str("bump.alloc(");
            self.generate_expression(&method_call.object)?;
            self.output.push(')');
        } else if method_call.method == "mutRef" && method_call.args.is_empty() {
            // Special case: obj.mutRef() becomes &mut obj
            // No automatic cloning - users must explicitly call .clone() if needed
            self.output.push_str("&mut ");
            self.generate_expression(&method_call.object)?;
        } else if method_call.method == "clone" && method_call.args.is_empty() {
            // Special case: obj.clone() becomes Clone::clone(obj) using UFCS
            // This avoids auto-ref and makes borrowing explicit
            self.output.push_str("Clone::clone(");
            self.generate_expression(&method_call.object)?;
            self.output.push(')');
        } else if method_call.method == "toString" && method_call.args.is_empty() {
            // Special case: obj.toString() becomes ToString::to_string(obj) using UFCS
            // This is pre-imported like clone
            self.output.push_str("ToString::to_string(");
            self.generate_expression(&method_call.object)?;
            self.output.push(')');
        } else {
            // Method requires import but wasn't imported
            return Err(CodegenError::MissingImport {
                method: method_call.method.clone(),
                type_name: "Type".to_string(), // We don't have the exact type here
                location: expr_span.start.clone(),
            }
            .into());
        }

        // Note: Method call comments are now handled by the statement generator to ensure proper placement after semicolons
        Ok(())
    }

    // Helper to collect all comments from a method chain
    pub(super) fn collect_method_chain_comments(&self, expr: &LocatedExpr) -> Vec<(String, String)> {
        let mut comments = Vec::new();

        if let Expr::MethodCall(method_call) = &expr.node {
            // First collect comments from the inner expression
            comments.extend(self.collect_method_chain_comments(&method_call.object));

            // Then add this method's comment if it exists
            if let Some(comment) = &method_call.inline_comment {
                comments.push(comment.clone());
            }
        }

        comments
    }
}
