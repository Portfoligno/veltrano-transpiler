use crate::ast::*;
use crate::builtins::BuiltinRegistry;
use crate::rust_interop::RustInteropRegistry;
use crate::types::*;

/// Type checking errors with detailed information
#[derive(Debug)]
pub enum TypeCheckError {
    TypeMismatch {
        expected: VeltranoType,
        actual: VeltranoType,
        location: SourceLocation,
    },
    TypeMismatchWithSuggestion {
        expected: VeltranoType,
        actual: VeltranoType,
        location: SourceLocation,
        suggestion: String,
    },
    MethodNotFound {
        receiver_type: VeltranoType,
        method: String,
        location: SourceLocation,
    },
    MethodNotFoundWithSuggestion {
        receiver_type: VeltranoType,
        method: String,
        location: SourceLocation,
        suggestion: String,
    },
    FieldNotFound {
        object_type: VeltranoType,
        field: String,
        location: SourceLocation,
    },
    FieldNotFoundWithSuggestion {
        object_type: VeltranoType,
        field: String,
        location: SourceLocation,
        suggestion: String,
    },
    ArgumentCountMismatch {
        function: String,
        expected: usize,
        actual: usize,
        location: SourceLocation,
    },
    IndexingNotSupported {
        object_type: VeltranoType,
        index_type: VeltranoType,
        location: SourceLocation,
    },
    BinaryOperatorNotSupported {
        operator: BinaryOp,
        left_type: VeltranoType,
        right_type: VeltranoType,
        location: SourceLocation,
    },
    VariableNotFound {
        name: String,
        location: SourceLocation,
    },
    FunctionNotFound {
        name: String,
        location: SourceLocation,
    },
    InvalidTypeConstructor {
        message: String,
        location: SourceLocation,
    },
}

/// Main type checker with strict type checking (no implicit conversions)
pub struct VeltranoTypeChecker {
    env: TypeEnvironment,
    trait_checker: RustInteropRegistry,
    builtin_registry: BuiltinRegistry,
}

/// Helper functions for trait checking on VeltranoType
impl VeltranoType {
    /// Check if this type implements Copy trait (should be naturally owned/value types)
    pub fn implements_copy(&self, trait_checker: &mut RustInteropRegistry) -> bool {
        // Only base types (no type arguments) can implement Copy directly
        if !self.args.is_empty() {
            return false;
        }

        let rust_type_name = self.to_rust_type_name();

        // Use the trait checker to determine Copy implementation
        trait_checker
            .type_implements_trait(&rust_type_name, "Copy")
            .unwrap_or(false)
    }

    /// Validate if Own<T> type constructor is valid with the given inner type
    pub fn validate_own_constructor(
        inner: &VeltranoType,
        trait_checker: &mut RustInteropRegistry,
    ) -> Result<(), String> {
        // Check if the inner type implements Copy (is naturally owned)
        let is_copy = inner.implements_copy(trait_checker);

        if is_copy {
            return Err(format!(
                "Cannot use Own<{:?}>. Types that implement Copy are always owned by default and don't need the Own<> wrapper.",
                inner.constructor
            ));
        }

        // Check for specific invalid combinations
        match &inner.constructor {
            TypeConstructor::MutRef => {
                Err("Cannot use Own<MutRef<T>>. MutRef<T> is already owned.".to_string())
            }
            TypeConstructor::Box => {
                Err("Cannot use Own<Box<T>>. Box<T> is already owned.".to_string())
            }
            TypeConstructor::Own => {
                Err("Cannot use Own<Own<T>>. This creates double ownership.".to_string())
            }
            _ => Ok(()),
        }
    }

    /// Check if this can be cloned (integrated with trait system)
    pub fn can_clone(&self, trait_checker: &mut RustInteropRegistry) -> bool {
        // For base types (no type arguments), use trait checker
        if self.args.is_empty() {
            let rust_type_name = self.to_rust_type_name();
            return trait_checker
                .type_implements_trait(&rust_type_name, "Clone")
                .unwrap_or(false);
        }

        // For complex types, assume they can be cloned if their base can be cloned
        // This is a simplification - in reality we'd need to check all type parameters
        let base = self.get_base_constructor();
        match base {
            TypeConstructor::I32
            | TypeConstructor::I64
            | TypeConstructor::ISize
            | TypeConstructor::U32
            | TypeConstructor::U64
            | TypeConstructor::USize
            | TypeConstructor::Bool
            | TypeConstructor::Char
            | TypeConstructor::Unit
            | TypeConstructor::Nothing => true,
            TypeConstructor::String => true,
            TypeConstructor::Str => false, // &str is Copy, not Clone
            TypeConstructor::Custom(_) => true, // Assume custom types can be cloned
            // For composed types (Vec, Array, etc.), assume they can be cloned
            _ => true,
        }
    }

    /// Check if this can be converted to string (integrated with trait system)
    pub fn can_to_string(&self, trait_checker: &mut RustInteropRegistry) -> bool {
        // For base types (no type arguments), use trait checker
        if self.args.is_empty() {
            let rust_type_name = self.to_rust_type_name();
            return trait_checker
                .type_implements_trait(&rust_type_name, "ToString")
                .unwrap_or(false);
        }

        // For complex types, assume they can be converted to string if their base can
        // This is a simplification - in reality we'd need to check all type parameters
        let base = self.get_base_constructor();
        match base {
            TypeConstructor::I32
            | TypeConstructor::I64
            | TypeConstructor::ISize
            | TypeConstructor::U32
            | TypeConstructor::U64
            | TypeConstructor::USize
            | TypeConstructor::Bool
            | TypeConstructor::Char
            | TypeConstructor::Unit
            | TypeConstructor::Nothing => true,
            TypeConstructor::String | TypeConstructor::Str => true,
            TypeConstructor::Custom(_) => true, // Assume custom types can be converted to string
            // For composed types, assume they can be converted to string
            _ => true,
        }
    }
}

impl VeltranoTypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            env: TypeEnvironment::new(),
            trait_checker: RustInteropRegistry::new(),
            builtin_registry: BuiltinRegistry::new(),
        };

        // Initialize built-in functions and methods
        checker.init_builtin_functions();
        checker
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

        for statement in &program.statements {
            if let Err(error) = self.check_statement(statement) {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check a statement for type correctness
    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), TypeCheckError> {
        match stmt {
            Stmt::VarDecl(var_decl, _) => self.check_var_declaration(var_decl),
            Stmt::FunDecl(fun_decl) => self.check_function_declaration(fun_decl),
            Stmt::Expression(expr, _) => {
                self.check_expression(expr)?;
                Ok(())
            }
            Stmt::Return(expr_opt, _) => {
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
            Stmt::Comment(_) | Stmt::Import(_) | Stmt::DataClass(_) => {
                // These don't need type checking
                Ok(())
            }
        }
    }

    /// Validate a type recursively, checking for invalid type constructor usage
    fn validate_type(&mut self, veltrano_type: &VeltranoType) -> Result<(), TypeCheckError> {
        match &veltrano_type.constructor {
            TypeConstructor::Own => {
                // Validate Own<T> type constructor
                if let Some(inner) = veltrano_type.inner() {
                    // First validate the inner type recursively
                    self.validate_type(inner)?;

                    // Then validate the Own<T> constraint
                    if let Err(err_msg) =
                        VeltranoType::validate_own_constructor(inner, &mut self.trait_checker)
                    {
                        return Err(TypeCheckError::InvalidTypeConstructor {
                            message: err_msg,
                            location: SourceLocation {
                                file: "unknown".to_string(),
                                line: 0,
                                column: 0,
                                source_line: "".to_string(),
                            },
                        });
                    }
                } else {
                    return Err(TypeCheckError::InvalidTypeConstructor {
                        message: "Own<T> requires a type parameter".to_string(),
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
                    });
                }
            }
            _ => {
                // For other type constructors, recursively validate type arguments
                for arg in &veltrano_type.args {
                    self.validate_type(arg)?;
                }
            }
        }
        Ok(())
    }

    /// Check variable declaration
    fn check_var_declaration(&mut self, var_decl: &VarDeclStmt) -> Result<(), TypeCheckError> {
        // Validate type annotation if present
        if let Some(declared_type) = &var_decl.type_annotation {
            self.validate_type(declared_type)?;
        }

        if let Some(initializer) = &var_decl.initializer {
            let init_type = self.check_expression(initializer)?;

            if let Some(declared_type) = &var_decl.type_annotation {
                let expected_type = declared_type.clone();

                // Strict type checking: types must match exactly
                if !self.types_equal(&expected_type, &init_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_type,
                        actual: init_type,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
                    });
                }
            }

            // Declare the variable in the environment
            self.env.declare_variable(var_decl.name.clone(), init_type);
        }

        Ok(())
    }

    /// Check function declaration
    fn check_function_declaration(&mut self, fun_decl: &FunDeclStmt) -> Result<(), TypeCheckError> {
        // Validate parameter types
        for param in &fun_decl.params {
            self.validate_type(&param.param_type)?;
        }

        // Validate return type if present
        if let Some(return_type) = &fun_decl.return_type {
            self.validate_type(return_type)?;
        }

        // Create function signature and add to environment
        let param_types: Vec<VeltranoType> = fun_decl
            .params
            .iter()
            .map(|p| p.param_type.clone())
            .collect();

        let return_type = fun_decl
            .return_type
            .as_ref()
            .cloned()
            .unwrap_or_else(|| VeltranoType::unit());

        let signature = FunctionSignature {
            name: fun_decl.name.clone(),
            parameters: param_types,
            return_type,
        };

        self.env.declare_function(fun_decl.name.clone(), signature);

        // Check function body
        self.env.enter_scope();

        // Add parameters to scope
        for param in &fun_decl.params {
            self.env
                .declare_variable(param.name.clone(), param.param_type.clone());
        }

        self.check_statement(&fun_decl.body)?;

        self.env.exit_scope();

        Ok(())
    }

    /// Check expression and return its type
    fn check_expression(&mut self, expr: &Expr) -> Result<VeltranoType, TypeCheckError> {
        match expr {
            Expr::Literal(literal) => self.check_literal(literal),
            Expr::Identifier(name) => self.check_identifier(name),
            Expr::Binary(binary) => self.check_binary_expression(binary),
            Expr::Unary(unary) => self.check_unary_expression(unary),
            Expr::Call(call) => self.check_call_expression(call),
            Expr::MethodCall(method_call) => self.check_method_call(method_call),
            Expr::FieldAccess(field_access) => self.check_field_access(field_access),
        }
    }

    /// Check literal expression
    fn check_literal(&self, literal: &LiteralExpr) -> Result<VeltranoType, TypeCheckError> {
        let veltrano_type = match literal {
            LiteralExpr::Int(_) => VeltranoType::i64(),
            LiteralExpr::Bool(_) => VeltranoType::bool(),
            LiteralExpr::String(_) => VeltranoType::string(), // String literals are naturally referenced
            LiteralExpr::Unit => VeltranoType::unit(),
            LiteralExpr::Null => VeltranoType::unit(), // For now, map null to unit
        };

        Ok(veltrano_type)
    }

    /// Check identifier (variable lookup)
    fn check_identifier(&self, name: &str) -> Result<VeltranoType, TypeCheckError> {
        self.env
            .lookup_variable(name)
            .cloned()
            .ok_or_else(|| TypeCheckError::VariableNotFound {
                name: name.to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            })
    }

    /// Check binary expression
    fn check_binary_expression(
        &mut self,
        binary: &BinaryExpr,
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

                if !self.types_equal(&left_type, &expected_int) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_int,
                        actual: left_type,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
                    });
                }

                if !self.types_equal(&right_type, &expected_int) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_int,
                        actual: right_type,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
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
                if !self.types_equal(&left_type, &right_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: left_type,
                        actual: right_type,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
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
    ) -> Result<VeltranoType, TypeCheckError> {
        let operand_type = self.check_expression(&unary.operand)?;

        match unary.operator {
            UnaryOp::Minus => {
                // Must be I64
                let expected_int = VeltranoType::i64();

                if !self.types_equal(&operand_type, &expected_int) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_int,
                        actual: operand_type,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
                    });
                }

                Ok(VeltranoType::i64())
            }
        }
    }

    /// Check function call expression
    fn check_call_expression(&mut self, call: &CallExpr) -> Result<VeltranoType, TypeCheckError> {
        if let Expr::Identifier(func_name) = call.callee.as_ref() {
            // Check if this is a built-in function first
            if self.builtin_registry.is_rust_macro(func_name) {
                return self.check_rust_macro_call(func_name, call);
            }

            // Check user-defined functions
            let func_sig = self
                .env
                .lookup_function(func_name)
                .cloned()
                .ok_or_else(|| TypeCheckError::FunctionNotFound {
                    name: func_name.clone(),
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                        source_line: "".to_string(),
                    },
                })?;

            // Check argument count
            if call.args.len() != func_sig.parameters.len() {
                return Err(TypeCheckError::ArgumentCountMismatch {
                    function: func_name.clone(),
                    expected: func_sig.parameters.len(),
                    actual: call.args.len(),
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                        source_line: "".to_string(),
                    },
                });
            }

            // Check argument types with strict matching
            for (_i, (arg, expected_param)) in
                call.args.iter().zip(&func_sig.parameters).enumerate()
            {
                let actual_param = match arg {
                    Argument::Bare(expr, _) => self.check_expression(expr)?,
                    Argument::Named(_, expr, _) => self.check_expression(expr)?,
                    Argument::Shorthand(_, _) => continue, // Type checking happens in codegen via variable lookup
                    Argument::StandaloneComment(_, _) => continue, // Skip comments
                };

                if !self.types_equal(expected_param, &actual_param) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: expected_param.clone(),
                        actual: actual_param,
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
                    });
                }
            }

            Ok(func_sig.return_type.clone())
        } else {
            // For now, only support direct function calls
            Err(TypeCheckError::FunctionNotFound {
                name: "unknown".to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            })
        }
    }

    /// Check Rust macro call (skip type checking)
    fn check_rust_macro_call(
        &mut self,
        _func_name: &str,
        call: &CallExpr,
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

    /// Check method call expression
    fn check_method_call(
        &mut self,
        method_call: &MethodCallExpr,
    ) -> Result<VeltranoType, TypeCheckError> {
        let receiver_type = self.check_expression(&method_call.object)?;

        // Check if this is a built-in method using the registry
        if let Some(return_type) = self.builtin_registry.get_method_return_type(
            &method_call.method,
            &receiver_type,
            &mut self.trait_checker,
        ) {
            return Ok(return_type);
        }

        // If not found in builtin registry, return method not found error
        Err(TypeCheckError::MethodNotFound {
            receiver_type,
            method: method_call.method.clone(),
            location: SourceLocation {
                file: "unknown".to_string(),
                line: 0,
                column: 0,
                source_line: "".to_string(),
            },
        })
    }

    /// Check field access expression
    fn check_field_access(
        &mut self,
        field_access: &FieldAccessExpr,
    ) -> Result<VeltranoType, TypeCheckError> {
        let object_type = self.check_expression(&field_access.object)?;

        // For now, return field not found (we'd need data class definitions)
        Err(TypeCheckError::FieldNotFound {
            object_type,
            field: field_access.field.clone(),
            location: SourceLocation {
                file: "unknown".to_string(),
                line: 0,
                column: 0,
                source_line: "".to_string(),
            },
        })
    }

    /// Core type equality check - no implicit conversion logic
    fn types_equal(&self, a: &VeltranoType, b: &VeltranoType) -> bool {
        a == b // Simple structural equality
    }
}

/// Error analyzer for providing conversion suggestions
pub struct ErrorAnalyzer;

impl ErrorAnalyzer {
    pub fn enhance_error(&self, error: TypeCheckError) -> TypeCheckError {
        match error {
            TypeCheckError::TypeMismatch {
                expected,
                actual,
                location,
            } => {
                if let Some(suggestion) = self.suggest_type_conversion(&expected, &actual) {
                    TypeCheckError::TypeMismatchWithSuggestion {
                        expected,
                        actual,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::TypeMismatch {
                        expected,
                        actual,
                        location,
                    }
                }
            }
            TypeCheckError::MethodNotFound {
                receiver_type,
                method,
                location,
            } => {
                if let Some(suggestion) = self.suggest_method_conversion(&receiver_type, &method) {
                    TypeCheckError::MethodNotFoundWithSuggestion {
                        receiver_type,
                        method,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::MethodNotFound {
                        receiver_type,
                        method,
                        location,
                    }
                }
            }
            TypeCheckError::FieldNotFound {
                object_type,
                field,
                location,
            } => {
                if let Some(suggestion) = self.suggest_field_conversion(&object_type, &field) {
                    TypeCheckError::FieldNotFoundWithSuggestion {
                        object_type,
                        field,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::FieldNotFound {
                        object_type,
                        field,
                        location,
                    }
                }
            }
            // Pass through other error types unchanged
            other => other,
        }
    }

    /// Suggest conversion from actual type to expected type
    fn suggest_type_conversion(
        &self,
        expected: &VeltranoType,
        actual: &VeltranoType,
    ) -> Option<String> {
        // Handle common conversion patterns with new type system

        // Pattern 1: Own<T> to T (remove ownership) -> .ref()
        if actual.constructor == TypeConstructor::Own {
            if let Some(inner) = actual.inner() {
                if inner == expected {
                    return Some(".ref()".to_string());
                }
            }
        }

        // Pattern 2: MutRef<T> to Ref<T> -> .ref()
        if expected.constructor == TypeConstructor::Ref
            && actual.constructor == TypeConstructor::MutRef
        {
            if let (Some(expected_inner), Some(actual_inner)) = (expected.inner(), actual.inner()) {
                if expected_inner == actual_inner {
                    return Some(".ref()".to_string());
                }
            }
        }

        // Pattern 5: Vec<T> to slice conversion -> .toSlice()
        if actual.constructor == TypeConstructor::Vec
            && expected.constructor == TypeConstructor::Ref
        {
            if let (Some(expected_inner), Some(actual_inner)) = (expected.inner(), actual.inner()) {
                if expected_inner == actual_inner {
                    return Some(".toSlice()".to_string());
                }
            }
        }

        // Pattern 6: Own<Vec<T>> to slice -> .ref().toSlice()
        if expected.constructor == TypeConstructor::Ref
            && actual.constructor == TypeConstructor::Own
        {
            if let Some(actual_inner) = actual.inner() {
                if actual_inner.constructor == TypeConstructor::Vec {
                    if let (Some(expected_inner), Some(vec_inner)) =
                        (expected.inner(), actual_inner.inner())
                    {
                        if expected_inner == vec_inner {
                            return Some(".ref().toSlice()".to_string());
                        }
                    }
                }
            }
        }

        // Pattern 7: Array to slice conversion
        if expected.constructor == TypeConstructor::Ref {
            if let TypeConstructor::Array(_) = actual.constructor {
                if let (Some(expected_inner), Some(actual_inner)) =
                    (expected.inner(), actual.inner())
                {
                    if expected_inner == actual_inner {
                        return Some(".toSlice()".to_string());
                    }
                }
            }
        }

        // Pattern 8: Own<Array<T, N>> to slice -> .ref().toSlice()
        if expected.constructor == TypeConstructor::Ref
            && actual.constructor == TypeConstructor::Own
        {
            if let Some(actual_inner) = actual.inner() {
                if let TypeConstructor::Array(_) = actual_inner.constructor {
                    if let (Some(expected_inner), Some(array_inner)) =
                        (expected.inner(), actual_inner.inner())
                    {
                        if expected_inner == array_inner {
                            return Some(".ref().toSlice()".to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Suggest method call with proper ownership conversion
    fn suggest_method_conversion(
        &self,
        receiver_type: &VeltranoType,
        method: &str,
    ) -> Option<String> {
        // Common pattern: owned types need .ref() before calling borrowed methods
        if receiver_type.constructor == TypeConstructor::Own {
            // Suggest adding .ref() before the method call for owned types
            Some(format!(".ref().{}()", method))
        } else {
            None
        }
    }

    /// Suggest field access with proper ownership conversion
    fn suggest_field_conversion(&self, object_type: &VeltranoType, field: &str) -> Option<String> {
        // Common pattern: owned types need .ref() before field access
        if object_type.constructor == TypeConstructor::Own {
            // Suggest adding .ref() before the field access for owned types
            Some(format!(".ref().{}", field))
        } else {
            None
        }
    }
}
