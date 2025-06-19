//! Expression checking logic for the type checker
//!
//! This module contains logic for checking various types of expressions
//! including literals, identifiers, binary/unary operations, function calls,
//! and field access.

use crate::ast::*;
use crate::error::{SourceLocation, Span};
use crate::types::{DataClassDefinition, FunctionSignature, TypeConstructor, VeltranoType};

use super::error::TypeCheckError;
use super::types::{substitute_generic_type, TypeValidator};
use super::VeltranoTypeChecker;

impl VeltranoTypeChecker {
    /// Helper to filter out standalone comment arguments
    fn filter_non_comment_args(args: &[Argument]) -> Vec<&Argument> {
        args.iter()
            .filter(|arg| !matches!(arg, Argument::StandaloneComment(_, _)))
            .collect()
    }

    /// Check expression with an optional expected type for inference
    pub(super) fn check_expression_with_expected_type(
        &mut self,
        expr: &LocatedExpr,
        expected_type: Option<&VeltranoType>,
    ) -> Result<VeltranoType, TypeCheckError> {
        match &expr.node {
            Expr::MethodCall(method_call) => {
                self.check_method_call_with_expected_type(method_call, expected_type)
            }
            _ => self.check_expression(expr),
        }
    }

    /// Check expression and return its type
    pub(super) fn check_expression(
        &mut self,
        expr: &LocatedExpr,
    ) -> Result<VeltranoType, TypeCheckError> {
        match &expr.node {
            Expr::Literal(literal) => self.check_literal(literal),
            Expr::Identifier(name) => self.check_identifier(name, &expr.span),
            Expr::Binary(binary) => self.check_binary_expression(binary, &expr.span),
            Expr::Unary(unary) => self.check_unary_expression(unary, &expr.span),
            Expr::Call(call) => self.check_call_expression(call, &expr.span),
            Expr::MethodCall(method_call) => self.check_method_call(method_call),
            Expr::FieldAccess(field_access) => self.check_field_access(field_access, &expr.span),
            Expr::Parenthesized(paren_expr) => self.check_expression(&paren_expr.expr),
        }
    }

    /// Check literal expression
    fn check_literal(&self, literal: &LiteralExpr) -> Result<VeltranoType, TypeCheckError> {
        let veltrano_type = match literal {
            LiteralExpr::Int(_) => VeltranoType::i64(),
            LiteralExpr::Bool(_) => VeltranoType::bool(),
            LiteralExpr::String(_) => VeltranoType::str(), // String literals have type Str
            LiteralExpr::Unit => VeltranoType::unit(),
            LiteralExpr::Null => VeltranoType::unit(), // For now, map null to unit
        };

        Ok(veltrano_type)
    }

    /// Check identifier (variable lookup)
    fn check_identifier(&self, name: &str, span: &Span) -> Result<VeltranoType, TypeCheckError> {
        self.env
            .lookup_variable(name)
            .cloned()
            .ok_or_else(|| TypeCheckError::VariableNotFound {
                name: name.to_string(),
                location: SourceLocation::new(span.start_line(), span.start_column()),
            })
    }

    /// Check binary expression
    fn check_binary_expression(
        &mut self,
        binary: &BinaryExpr,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        let left_type = self.check_expression(&binary.left)?;
        let right_type = self.check_expression(&binary.right)?;

        // For now, implement basic arithmetic and comparison operators
        match binary.operator {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                // Both operands must be I64
                let expected_int = VeltranoType::i64();

                if !TypeValidator::types_equal(&left_type, &expected_int) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_int,
                        actual: left_type,
                        location: SourceLocation::new(
                            binary.left.span.start_line(),
                            binary.left.span.start_column(),
                        ),
                    });
                }

                if !TypeValidator::types_equal(&right_type, &expected_int) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_int,
                        actual: right_type,
                        location: SourceLocation::new(
                            binary.right.span.start_line(),
                            binary.right.span.start_column(),
                        ),
                    });
                }

                Ok(VeltranoType::i64())
            }
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => {
                // Types must match exactly, result is Bool
                if !TypeValidator::types_equal(&left_type, &right_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: left_type,
                        actual: right_type,
                        location: SourceLocation::new(span.start_line(), span.start_column()),
                    });
                }

                Ok(VeltranoType::bool())
            }
            BinaryOp::And | BinaryOp::Or => {
                // Both operands must be Bool
                let expected_bool = VeltranoType::bool();

                if !TypeValidator::types_equal(&left_type, &expected_bool) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_bool,
                        actual: left_type,
                        location: SourceLocation::new(
                            binary.left.span.start_line(),
                            binary.left.span.start_column(),
                        ),
                    });
                }

                if !TypeValidator::types_equal(&right_type, &expected_bool) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_bool,
                        actual: right_type,
                        location: SourceLocation::new(
                            binary.right.span.start_line(),
                            binary.right.span.start_column(),
                        ),
                    });
                }

                Ok(VeltranoType::bool())
            }
        }
    }

    /// Check unary expression
    fn check_unary_expression(
        &mut self,
        unary: &UnaryExpr,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        let operand_type = self.check_expression(&unary.operand)?;

        match unary.operator {
            UnaryOp::Minus => {
                // Must be I64
                let expected_int = VeltranoType::i64();

                if !TypeValidator::types_equal(&operand_type, &expected_int) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_int,
                        actual: operand_type,
                        location: SourceLocation::new(span.start_line(), span.start_column()),
                    });
                }

                Ok(VeltranoType::i64())
            }
        }
    }

    /// Check function call expression
    pub(super) fn check_call_expression(
        &mut self,
        call: &CallExpr,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        if let Expr::Identifier(func_name) = &call.callee.node {
            // Check if this is a built-in function first
            if self.builtin_registry.is_rust_macro(func_name) {
                return self.check_rust_macro_call(func_name, call, span);
            }

            // Check if this is a data class constructor
            if let Some(data_class) = self.env.lookup_data_class(func_name).cloned() {
                return self.check_data_class_constructor_call(func_name, &data_class, call, span);
            }

            // Check user-defined functions first (highest priority)
            if let Some(func_sig) = self.env.lookup_function(func_name).cloned() {
                // Check argument count (excluding standalone comments)
                let non_comment_args = Self::filter_non_comment_args(&call.args);
                let actual_arg_count = non_comment_args.len();

                if actual_arg_count != func_sig.parameters.len() {
                    return Err(TypeCheckError::ArgumentCountMismatch {
                        function: func_name.clone(),
                        expected: func_sig.parameters.len(),
                        actual: actual_arg_count,
                        location: SourceLocation::new(span.start_line(), span.start_column()),
                    });
                }

                // Check if this is a generic function
                let has_generic_params = func_sig
                    .parameters
                    .iter()
                    .any(|p| matches!(&p.constructor, TypeConstructor::Generic(_, _)));

                if has_generic_params {
                    // Handle generic function instantiation
                    return self.check_generic_function_call(func_name, &func_sig, call, span);
                }

                // Type check arguments
                for (i, arg) in non_comment_args.iter().enumerate() {
                    let arg_expr = match arg {
                        Argument::Bare(expr, _) => expr,
                        Argument::Named(name, _, _) => {
                            return Err(TypeCheckError::UnsupportedFeature {
                                feature: format!("Named argument '{}'", name),
                                location: SourceLocation::new(
                                    span.start_line(),
                                    span.start_column(),
                                ),
                            });
                        }
                        Argument::Shorthand(field, _) => {
                            return Err(TypeCheckError::UnsupportedFeature {
                                feature: format!("Shorthand argument '.{}'", field),
                                location: SourceLocation::new(
                                    span.start_line(),
                                    span.start_column(),
                                ),
                            });
                        }
                        Argument::StandaloneComment(_, _) => unreachable!(), // filtered out
                    };

                    let expected_type = &func_sig.parameters[i];
                    let actual_type =
                        self.check_expression_with_expected_type(arg_expr, Some(expected_type))?;

                    if &actual_type != expected_type {
                        return Err(TypeCheckError::TypeMismatch {
                            expected: expected_type.clone(),
                            actual: actual_type,
                            location: SourceLocation::new(
                                arg_expr.span.start_line(),
                                arg_expr.span.start_column(),
                            ),
                        });
                    }
                }

                return Ok(func_sig.return_type.clone());
            }

            // Check if this is an imported method being called as a function
            if let Some(_imports) = self.import_handler.get_imports(func_name) {
                // For standalone method calls, we need to handle them specially
                // This handles cases like `newVec()` where `newVec` is an alias for `Vec.new`
                return self.import_handler.check_standalone_method_call(
                    func_name,
                    call,
                    &mut self.trait_checker,
                    &mut self.method_resolutions,
                    &span,
                );
            }

            // Function not found in any scope
            Err(TypeCheckError::FunctionNotFound {
                name: func_name.clone(),
                location: SourceLocation::new(span.start_line(), span.start_column()),
            })
        } else {
            // For now, only support direct function calls
            Err(TypeCheckError::FunctionNotFound {
                name: "unknown".to_string(),
                location: SourceLocation::new(span.start_line(), span.start_column()),
            })
        }
    }

    /// Check generic function call by instantiating type parameters
    fn check_generic_function_call(
        &mut self,
        func_name: &str,
        func_sig: &FunctionSignature,
        call: &CallExpr,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        // For now, we only support single-parameter generic functions
        let non_comment_args = Self::filter_non_comment_args(&call.args);
        let actual_arg_count = non_comment_args.len();

        if actual_arg_count != 1 || func_sig.parameters.len() != 1 {
            return Err(TypeCheckError::ArgumentCountMismatch {
                function: func_name.to_string(),
                expected: func_sig.parameters.len(),
                actual: actual_arg_count,
                location: SourceLocation::new(span.start_line(), span.start_column()),
            });
        }

        // Check the argument type (skip standalone comments)
        let first_non_comment_arg = non_comment_args[0]; // Safe because we already checked count

        let arg_type = match first_non_comment_arg {
            Argument::Bare(expr, _) => self.check_expression(expr)?,
            Argument::Named(_, _, _) | Argument::Shorthand(_, _) => {
                return Err(TypeCheckError::ArgumentCountMismatch {
                    function: func_name.to_string(),
                    expected: 1,
                    actual: actual_arg_count,
                    location: SourceLocation::new(span.start_line(), span.start_column()),
                });
            }
            Argument::StandaloneComment(_, _) => {
                // This should never happen because we filtered out comments
                unreachable!("StandaloneComment should have been filtered out")
            }
        };

        // Get the generic parameter
        if let TypeConstructor::Generic(param_name, _constraints) =
            &func_sig.parameters[0].constructor
        {
            // Substitute the generic type in the return type
            let return_type = substitute_generic_type(&func_sig.return_type, param_name, &arg_type);
            Ok(return_type)
        } else {
            // This shouldn't happen if we detected generics correctly
            Err(TypeCheckError::FunctionNotFound {
                name: func_name.to_string(),
                location: SourceLocation::new(span.start_line(), span.start_column()),
            })
        }
    }

    /// Check data class constructor call
    fn check_data_class_constructor_call(
        &mut self,
        class_name: &str,
        data_class: &DataClassDefinition,
        call: &CallExpr,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Filter out comments to get actual arguments
        let actual_args = Self::filter_non_comment_args(&call.args);

        // Check argument count
        if actual_args.len() != data_class.fields.len() {
            return Err(TypeCheckError::ArgumentCountMismatch {
                function: class_name.to_string(),
                expected: data_class.fields.len(),
                actual: actual_args.len(),
                location: SourceLocation::new(span.start_line(), span.start_column()),
            });
        }

        // Check argument types, handling named arguments and field order
        for (i, arg) in actual_args.iter().enumerate() {
            let (expected_field, actual_type, arg_span) = match arg {
                Argument::Bare(expr, _) => {
                    // Positional argument - match by index
                    let field = &data_class.fields[i];
                    let actual_type = self.check_expression(expr)?;
                    (field, actual_type, expr.span.clone())
                }
                Argument::Named(field_name, expr, _) => {
                    // Named argument - find matching field
                    let field = data_class
                        .fields
                        .iter()
                        .find(|f| f.name == *field_name)
                        .ok_or_else(|| TypeCheckError::FieldNotFound {
                            object_type: VeltranoType {
                                constructor: TypeConstructor::Custom(class_name.to_string()),
                                args: vec![],
                            },
                            field: field_name.clone(),
                            location: SourceLocation::new(
                                expr.span.start_line(),
                                expr.span.start_column(),
                            ),
                        })?;
                    let actual_type = self.check_expression(expr)?;
                    (field, actual_type, expr.span.clone())
                }
                Argument::Shorthand(var_name, _) => {
                    // Shorthand argument - field name is the variable name
                    let field = data_class
                        .fields
                        .iter()
                        .find(|f| f.name == *var_name)
                        .ok_or_else(|| TypeCheckError::FieldNotFound {
                            object_type: VeltranoType {
                                constructor: TypeConstructor::Custom(class_name.to_string()),
                                args: vec![],
                            },
                            field: var_name.clone(),
                            location: SourceLocation::new(span.start_line(), span.start_column()),
                        })?;
                    // For shorthand, we need to create a dummy span
                    let actual_type = self.check_identifier(var_name, span)?;
                    (field, actual_type, span.clone())
                }
                Argument::StandaloneComment(_, _) => unreachable!(), // Filtered out above
            };

            // Check type compatibility
            if !TypeValidator::types_equal(&expected_field.field_type, &actual_type) {
                return Err(TypeCheckError::TypeMismatch {
                    expected: expected_field.field_type.clone(),
                    actual: actual_type,
                    location: SourceLocation::new(arg_span.start_line(), arg_span.start_column()),
                });
            }
        }

        // Return the data class type as owned
        Ok(VeltranoType::own(VeltranoType {
            constructor: TypeConstructor::Custom(class_name.to_string()),
            args: vec![],
        }))
    }

    /// Check Rust macro call (skip type checking)
    fn check_rust_macro_call(
        &mut self,
        _func_name: &str,
        call: &CallExpr,
        _span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Rust macros skip type checking - they accept any arguments
        // Just validate that arguments are syntactically correct expressions
        for arg in &call.args {
            match arg {
                Argument::Bare(expr, _) => {
                    self.check_expression(expr)?; // Ensure expression is valid
                }
                Argument::Named(_, expr, _) => {
                    self.check_expression(expr)?; // Ensure expression is valid
                }
                Argument::Shorthand(_, _) => continue, // No expression to validate
                Argument::StandaloneComment(_, _) => continue, // Skip comments
            }
        }

        // Return unit type for macros like println!, print!, panic!
        Ok(VeltranoType::unit())
    }

    /// Check field access expression
    pub(super) fn check_field_access(
        &mut self,
        field_access: &FieldAccessExpr,
        span: &Span,
    ) -> Result<VeltranoType, TypeCheckError> {
        let object_type = self.check_expression(&field_access.object)?;

        // Handle field access based on the object type
        match &object_type.constructor {
            TypeConstructor::Custom(class_name) => {
                // Look up the data class definition
                if let Some(data_class) = self.env.lookup_data_class(&class_name) {
                    // Find the field in the data class
                    if let Some(field_def) = data_class
                        .fields
                        .iter()
                        .find(|f| f.name == field_access.field)
                    {
                        return Ok(field_def.field_type.clone());
                    }
                }

                // Field not found in data class
                Err(TypeCheckError::FieldNotFound {
                    object_type,
                    field: field_access.field.clone(),
                    location: SourceLocation::new(span.start_line(), span.start_column()),
                })
            }
            TypeConstructor::Own => {
                // For Own<T>, allow direct field access on the inner type
                if let Some(inner_type) = object_type.inner() {
                    if let TypeConstructor::Custom(class_name) = &inner_type.constructor {
                        // Look up the data class definition
                        if let Some(data_class) = self.env.lookup_data_class(&class_name) {
                            // Find the field in the data class
                            if let Some(field_def) = data_class
                                .fields
                                .iter()
                                .find(|f| f.name == field_access.field)
                            {
                                return Ok(field_def.field_type.clone());
                            }
                        }
                    }
                }

                // Field not found
                Err(TypeCheckError::FieldNotFound {
                    object_type,
                    field: field_access.field.clone(),
                    location: SourceLocation::new(span.start_line(), span.start_column()),
                })
            }
            _ => {
                // Other types don't support field access
                Err(TypeCheckError::FieldNotFound {
                    object_type,
                    field: field_access.field.clone(),
                    location: SourceLocation::new(span.start_line(), span.start_column()),
                })
            }
        }
    }
}
