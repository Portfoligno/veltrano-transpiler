use crate::ast::*;
use std::collections::HashMap;

/// A type in the Veltrano type system supporting higher-kinded types
#[derive(Debug, Clone, PartialEq)]
pub struct VeltranoType {
    /// The type constructor or base type
    pub constructor: TypeConstructor,
    /// Type arguments (empty for base types)
    pub args: Vec<VeltranoType>,
}

/// Type constructors and base types with their kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstructor {
    // Base types (kind *)
    /// i64 in Rust
    I64,
    /// bool in Rust
    Bool,
    /// () in Rust
    Unit,
    /// ! in Rust (never type)
    Nothing,
    /// &str in Rust (string slice)
    Str,
    /// &String in Rust (reference to owned string)
    String,
    /// Custom/user-defined types
    Custom(String),

    // Built-in type constructors (kind * -> *)
    /// Own<T> - forces ownership, removes reference level for reference types
    Own,
    /// Ref<T> - adds reference level (&T)
    Ref,
    /// MutRef<T> - mutable reference (&mut T)
    MutRef,
    /// Box<T> - heap allocation
    Box,
    /// Vec<T> - dynamic array
    Vec,
    /// Option<T> - optional value
    Option,

    // Higher-kinded constructors (kind * -> * -> *)
    /// Result<T, E> - result type
    Result,

    // Special cases
    /// Array<T, N> - fixed-size array (size is part of type)
    Array(usize),
}

impl VeltranoType {
    /// Helper constructors for base types
    pub fn i64() -> Self {
        Self {
            constructor: TypeConstructor::I64,
            args: vec![],
        }
    }

    pub fn bool() -> Self {
        Self {
            constructor: TypeConstructor::Bool,
            args: vec![],
        }
    }

    pub fn unit() -> Self {
        Self {
            constructor: TypeConstructor::Unit,
            args: vec![],
        }
    }

    pub fn nothing() -> Self {
        Self {
            constructor: TypeConstructor::Nothing,
            args: vec![],
        }
    }

    pub fn str() -> Self {
        Self {
            constructor: TypeConstructor::Str,
            args: vec![],
        }
    }

    pub fn string() -> Self {
        Self {
            constructor: TypeConstructor::String,
            args: vec![],
        }
    }

    pub fn custom(name: String) -> Self {
        Self {
            constructor: TypeConstructor::Custom(name),
            args: vec![],
        }
    }

    /// Apply type constructors
    pub fn own(inner: VeltranoType) -> Self {
        // Note: Validation is now handled during type checking phase
        // This constructor only creates the type representation
        Self {
            constructor: TypeConstructor::Own,
            args: vec![inner],
        }
    }

    pub fn ref_(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Ref,
            args: vec![inner],
        }
    }

    pub fn mut_ref(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::MutRef,
            args: vec![inner],
        }
    }

    pub fn vec(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Vec,
            args: vec![inner],
        }
    }

    pub fn boxed(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Box,
            args: vec![inner],
        }
    }

    pub fn array(inner: VeltranoType, size: usize) -> Self {
        Self {
            constructor: TypeConstructor::Array(size),
            args: vec![inner],
        }
    }

    pub fn option(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Option,
            args: vec![inner],
        }
    }

    pub fn result(ok_type: VeltranoType, err_type: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Result,
            args: vec![ok_type, err_type],
        }
    }

    /// Compatibility methods for migration
    pub fn inner(&self) -> Option<&VeltranoType> {
        self.args.first()
    }

    pub fn inner_mut(&mut self) -> Option<&mut VeltranoType> {
        self.args.first_mut()
    }

    /// Check if this is a naturally owned type (Int, Bool, Unit, Nothing)
    pub fn is_naturally_owned(&self) -> bool {
        self.args.is_empty()
            && matches!(
                self.constructor,
                TypeConstructor::I64
                    | TypeConstructor::Bool
                    | TypeConstructor::Unit
                    | TypeConstructor::Nothing
            )
    }

    /// Check if this is a naturally referenced type (Str, String, Custom)
    pub fn is_naturally_referenced(&self) -> bool {
        self.args.is_empty()
            && matches!(
                self.constructor,
                TypeConstructor::Str | TypeConstructor::String | TypeConstructor::Custom(_)
            )
    }

    /// Validate if Own<T> type constructor is valid with the given inner type
    pub fn validate_own_constructor(inner: &VeltranoType) -> Result<(), String> {
        // Check if the inner type is naturally owned (Int, Bool, Unit, Nothing)
        if inner.is_naturally_owned() {
            return Err(format!(
                "Cannot use Own<{:?}>. This type is already owned.",
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

    /// Get the ultimate base type constructor (recursively unwrap constructors)
    pub fn get_base_constructor(&self) -> &TypeConstructor {
        if self.args.is_empty() {
            &self.constructor
        } else if let Some(inner) = self.inner() {
            inner.get_base_constructor()
        } else {
            &self.constructor
        }
    }

    /// Check if this can be cloned (TODO: integrate with trait system)
    pub fn can_clone(&self) -> bool {
        let base = self.get_base_constructor();
        match base {
            TypeConstructor::I64
            | TypeConstructor::Bool
            | TypeConstructor::Unit
            | TypeConstructor::Nothing => true,
            TypeConstructor::String => true,
            TypeConstructor::Str => false, // &str is Copy, not Clone
            TypeConstructor::Custom(_) => true, // Assume custom types can be cloned
            // For composed types (Vec, Array, etc.), assume they can be cloned
            _ => true,
        }
    }

    /// Check if this can be converted to string (TODO: integrate with trait system)
    pub fn can_to_string(&self) -> bool {
        let base = self.get_base_constructor();
        match base {
            TypeConstructor::I64
            | TypeConstructor::Bool
            | TypeConstructor::Unit
            | TypeConstructor::Nothing => true,
            TypeConstructor::String | TypeConstructor::Str => true,
            TypeConstructor::Custom(_) => true, // Assume custom types can be converted to string
            // For composed types, assume they can be converted to string
            _ => true,
        }
    }
}

/// Function signature for type checking
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<VeltranoType>,
    pub return_type: VeltranoType,
}

/// Method signature for type checking
#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub receiver_type: VeltranoType,
    pub parameters: Vec<VeltranoType>,
    pub return_type: VeltranoType,
}

/// Source location for error reporting
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
}

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

/// Type environment for tracking variables and functions
pub struct TypeEnvironment {
    variables: HashMap<String, VeltranoType>,
    functions: HashMap<String, FunctionSignature>,
    scopes: Vec<HashMap<String, VeltranoType>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn lookup_variable(&self, name: &str) -> Option<&VeltranoType> {
        // Check current scopes first (most recent first)
        for scope in self.scopes.iter().rev() {
            if let Some(var_type) = scope.get(name) {
                return Some(var_type);
            }
        }

        // Check global variables
        self.variables.get(name)
    }

    pub fn declare_variable(&mut self, name: String, typ: VeltranoType) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, typ);
        } else {
            self.variables.insert(name, typ);
        }
    }

    pub fn declare_function(&mut self, name: String, signature: FunctionSignature) {
        self.functions.insert(name, signature);
    }

    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }
}

/// Main type checker with strict type checking (no implicit conversions)
pub struct VeltranoTypeChecker {
    env: TypeEnvironment,
}

impl VeltranoTypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            env: TypeEnvironment::new(),
        };

        // Initialize built-in functions and methods
        checker.init_builtin_functions();
        checker
    }

    fn init_builtin_functions(&mut self) {
        // TODO: Register built-in function signatures from the builtin registry
        // For now, we'll just handle built-ins in the call checking logic
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
    fn validate_type(&self, veltrano_type: &VeltranoType) -> Result<(), TypeCheckError> {
        match &veltrano_type.constructor {
            TypeConstructor::Own => {
                // Validate Own<T> type constructor
                if let Some(inner) = veltrano_type.inner() {
                    // First validate the inner type recursively
                    self.validate_type(inner)?;

                    // Then validate the Own<T> constraint
                    if let Err(err_msg) = VeltranoType::validate_own_constructor(inner) {
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
            if self.is_rust_macro(func_name) {
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

    /// Check if a function is a Rust macro
    fn is_rust_macro(&self, name: &str) -> bool {
        matches!(
            name,
            "println" | "print" | "panic" | "assert" | "debug_assert"
        )
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

        // Handle built-in conversion methods
        match method_call.method.as_str() {
            "ref" => self.check_ref_method(&receiver_type),
            "mutRef" => self.check_mut_ref_method(&receiver_type),
            "toSlice" => self.check_to_slice_method(&receiver_type),
            "clone" => self.check_clone_method(&receiver_type),
            _ => {
                // Handle built-in methods
                self.check_builtin_method(&receiver_type, &method_call.method)
            }
        }
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

    /// Check .ref() method call (TODO: integrate with builtin registry)
    fn check_ref_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Implement correct ref() semantics:
        // Own<T> → T, T → Ref<T>, MutRef<T> → Ref<MutRef<T>>
        match &receiver_type.constructor {
            // Own<T> → T (remove the Own wrapper)
            TypeConstructor::Own => {
                if let Some(inner) = receiver_type.inner() {
                    Ok(inner.clone())
                } else {
                    // Shouldn't happen with well-formed Own<T>
                    Ok(VeltranoType::ref_(receiver_type.clone()))
                }
            }
            // T → Ref<T> (add a Ref wrapper)
            _ => Ok(VeltranoType::ref_(receiver_type.clone())),
        }
    }

    /// Check .mutRef() method call (TODO: integrate with builtin registry)
    fn check_mut_ref_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Implement correct mutRef() semantics:
        // Available on owned and mutable types: Own<T> → MutRef<Own<T>>, MutRef<T> → MutRef<MutRef<T>>
        match &receiver_type.constructor {
            // Own<T> → MutRef<Own<T>>
            TypeConstructor::Own => Ok(VeltranoType::mut_ref(receiver_type.clone())),
            // MutRef<T> → MutRef<MutRef<T>>
            TypeConstructor::MutRef => Ok(VeltranoType::mut_ref(receiver_type.clone())),
            // All other types should fail
            _ => Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: "mutRef".to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            }),
        }
    }

    /// Check .toSlice() method call (TODO: integrate with builtin registry)
    fn check_to_slice_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        // For now, just return a reference to the receiver
        // This should be handled by builtin registry
        Ok(VeltranoType::ref_(receiver_type.clone()))
    }

    /// Check .clone() method call (TODO: integrate with builtin registry)
    fn check_clone_method(
        &mut self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        if receiver_type.can_clone() {
            // Clone returns an owned version - use builtin registry logic instead
            if receiver_type.is_naturally_referenced() {
                Ok(VeltranoType::own(receiver_type.clone()))
            } else {
                Ok(receiver_type.clone()) // Already owned for value types
            }
        } else {
            Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: "clone".to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            })
        }
    }

    /// Convert VeltranoType to Rust type name for trait checking (TODO: use rust_interop)
    fn veltrano_type_to_rust_type_name(&self, veltrano_type: &VeltranoType) -> String {
        // For now, just return a placeholder
        // TODO: Implement proper type name generation for new type system
        let base = veltrano_type.get_base_constructor();
        match base {
            TypeConstructor::I64 => "i64".to_string(),
            TypeConstructor::Bool => "bool".to_string(),
            TypeConstructor::Str => "&str".to_string(),
            TypeConstructor::String => "String".to_string(),
            TypeConstructor::Unit => "()".to_string(),
            TypeConstructor::Nothing => "!".to_string(),
            TypeConstructor::Custom(name) => name.clone(),
            _ => "unknown".to_string(),
        }
    }

    /// Check built-in methods on types
    fn check_builtin_method(
        &mut self,
        receiver_type: &VeltranoType,
        method: &str,
    ) -> Result<VeltranoType, TypeCheckError> {
        match method {
            "toString" => self.check_tostring_method(receiver_type),
            "length" => self.check_length_method(receiver_type),
            _ => Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: method.to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            }),
        }
    }

    /// Check .toString() method call (TODO: integrate with builtin registry)
    fn check_tostring_method(
        &mut self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        if receiver_type.can_to_string() {
            // toString returns an owned String
            Ok(VeltranoType::own(VeltranoType::string()))
        } else {
            Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: "toString".to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            })
        }
    }

    /// Check .length() method call (TODO: integrate with builtin registry)
    fn check_length_method(
        &self,
        _receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        // For now, just return I64 for length method (should use builtin registry)
        Ok(VeltranoType::i64())
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

        // Pattern 3: String to Str conversion
        if expected.constructor == TypeConstructor::Str
            && actual.constructor == TypeConstructor::String
        {
            return Some(".ref()".to_string());
        }

        // Pattern 4: Own<String> to Str (remove ownership then convert to str)
        if expected.constructor == TypeConstructor::Str
            && actual.constructor == TypeConstructor::Own
        {
            if let Some(inner) = actual.inner() {
                if inner.constructor == TypeConstructor::String {
                    return Some(".ref().ref()".to_string());
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
