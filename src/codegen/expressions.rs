//! Expression code generation.
//!
//! Handles literals, operators, calls, and field access.

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
                self.generate_binary_expression(binary)?;
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

    /// Generate code for binary expressions with proper comment handling
    fn generate_binary_expression(&mut self, binary: &BinaryExpr) -> Result<(), VeltranoError> {
        self.generate_expression(&binary.left)?;
        
        // Generate comment after left operand if present
        self.generate_binary_operator_comment(&binary.comment_after_left);
        
        self.generate_binary_operator(&binary.operator);
        
        // Generate comment after operator if present
        self.generate_binary_operator_comment(&binary.comment_after_operator);
        
        self.generate_expression(&binary.right)?;
        Ok(())
    }

    /// Generate a comment between binary operator parts
    fn generate_binary_operator_comment(&mut self, comment: &Option<(String, String)>) {
        if let Some((content, whitespace)) = comment {
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
            let resolution = resolution.clone();
            return self.generate_resolved_import_call(&resolution, call, call_span);
        }

        if let Expr::Identifier(name) = &call.callee.node {
            // Check if this is a data class constructor
            if self.data_classes.contains(name) {
                return self.generate_data_class_constructor(name, call, call_span);
            }

            if name == "MutRef" {
                return self.generate_mutref_builtin(call, call_span);
            } else if self.local_functions.contains(name) {
                return self.generate_local_function_call(name, call, call_span);
            } else if let Some((type_name, original_method)) = self.imports.get(name) {
                let type_name = type_name.clone();
                let original_method = original_method.clone();
                return self.generate_imported_function_call(&type_name, &original_method, call, call_span);
            } else if self.is_rust_macro(name) {
                return self.generate_macro_call(name, call, call_span);
            }
        }
        
        // Default case: generate as generic call
        self.generate_generic_call(call, call_span)
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
            let resolution = resolution.clone();
            return self.generate_resolved_method_call(&resolution, method_call);
        }
        
        if let Some((type_name, original_method)) = self.imports.get(&method_call.method) {
            let type_name = type_name.clone();
            let original_method = original_method.clone();
            return self.generate_imported_method_call(&type_name, &original_method, method_call);
        }
        
        // Check for special builtin methods
        match method_call.method.as_str() {
            "ref" if method_call.args.is_empty() => {
                self.output.push('&');
                self.generate_expression(&method_call.object)?;
                Ok(())
            }
            "bumpRef" if method_call.args.is_empty() => {
                self.output.push_str("bump.alloc(");
                self.generate_expression(&method_call.object)?;
                self.output.push(')');
                Ok(())
            }
            "mutRef" if method_call.args.is_empty() => {
                self.output.push_str("&mut ");
                self.generate_expression(&method_call.object)?;
                Ok(())
            }
            "clone" if method_call.args.is_empty() => {
                self.output.push_str("Clone::clone(");
                self.generate_expression(&method_call.object)?;
                self.output.push(')');
                Ok(())
            }
            "toString" if method_call.args.is_empty() => {
                self.output.push_str("ToString::to_string(");
                self.generate_expression(&method_call.object)?;
                self.output.push(')');
                Ok(())
            }
            _ => {
                // Method requires import but wasn't imported
                Err(CodegenError::MissingImport {
                    method: method_call.method.clone(),
                    type_name: "Type".to_string(), // We don't have the exact type here
                    location: expr_span.start.clone(),
                }
                .into())
            }
        }
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

    /// Generate code for a resolved import call (e.g., newVec -> Vec::new)
    fn generate_resolved_import_call(
        &mut self,
        resolution: &crate::type_checker::MethodResolution,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
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
        Ok(())
    }

    /// Generate code for data class constructor calls
    fn generate_data_class_constructor(
        &mut self,
        name: &str,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
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
                    Argument::Named(field_name, expr, comment) => {
                        self.indent();
                        self.output.push_str(&camel_to_snake_case(field_name));
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
                            constructor: name.to_string(),
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

        Ok(())
    }

    /// Generate code for the MutRef builtin function
    fn generate_mutref_builtin(
        &mut self,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
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
        Ok(())
    }

    /// Generate code for locally defined function calls
    fn generate_local_function_call(
        &mut self,
        name: &str,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
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
        Ok(())
    }

    /// Generate code for imported function/constructor calls
    fn generate_imported_function_call(
        &mut self,
        type_name: &str,
        original_method: &str,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
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
        Ok(())
    }

    /// Generate code for Rust macro calls
    fn generate_macro_call(
        &mut self,
        name: &str,
        call: &CallExpr,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        self.output.push_str(name);
        self.output.push('!');
        self.output.push('(');
        self.generate_comma_separated_args_for_function_call_with_multiline(
            &call.args,
            call.is_multiline,
            call_span,
        )?;
        self.output.push(')');
        Ok(())
    }

    /// Generate code for a resolved method call
    fn generate_resolved_method_call(
        &mut self,
        resolution: &crate::type_checker::MethodResolution,
        method_call: &MethodCallExpr,
    ) -> Result<(), VeltranoError> {
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
        Ok(())
    }

    /// Generate code for an imported method call
    fn generate_imported_method_call(
        &mut self,
        type_name: &str,
        original_method: &str,
        method_call: &MethodCallExpr,
    ) -> Result<(), VeltranoError> {
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
        Ok(())
    }
}
