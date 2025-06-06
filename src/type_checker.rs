use crate::ast::*;
use std::collections::HashMap;

/// The Veltrano type representation for strict type checking
#[derive(Debug, Clone, PartialEq)]
pub struct VeltranoType {
    pub base: VeltranoBaseType,
    pub ownership: Ownership,
    pub mutability: Mutability,
}

/// Ownership levels in Veltrano's type system
#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Owned,       // Own<T> - equivalent to T in Rust
    Borrowed,    // T - equivalent to &T in Rust
    MutBorrowed, // MutRef<T> - equivalent to &mut T in Rust
}

/// Mutability specification
#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// Extended base types for Veltrano
#[derive(Debug, Clone, PartialEq)]
pub enum VeltranoBaseType {
    Int,
    Bool,
    Str,
    String,
    Unit,
    Vec(Box<VeltranoType>),          // Vec<T> - owned dynamic arrays
    Slice(Box<VeltranoType>),        // Slice<T> - borrowed array views (&[T] in Rust)
    Array(Box<VeltranoType>, usize), // [T; N] - fixed-size arrays
    Custom(String),
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

    /// Check variable declaration
    fn check_var_declaration(&mut self, var_decl: &VarDeclStmt) -> Result<(), TypeCheckError> {
        if let Some(initializer) = &var_decl.initializer {
            let init_type = self.check_expression(initializer)?;

            if let Some(declared_type) = &var_decl.type_annotation {
                let expected_type = self.convert_ast_type_to_veltrano_type(declared_type);

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
        // Create function signature and add to environment
        let param_types: Vec<VeltranoType> = fun_decl
            .params
            .iter()
            .map(|p| self.convert_ast_type_to_veltrano_type(&p.param_type))
            .collect();

        let return_type = fun_decl
            .return_type
            .as_ref()
            .map(|t| self.convert_ast_type_to_veltrano_type(t))
            .unwrap_or_else(|| VeltranoType {
                base: VeltranoBaseType::Unit,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            });

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
            let param_type = self.convert_ast_type_to_veltrano_type(&param.param_type);
            self.env.declare_variable(param.name.clone(), param_type);
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
            LiteralExpr::Int(_) => VeltranoType {
                base: VeltranoBaseType::Int,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            },
            LiteralExpr::Bool(_) => VeltranoType {
                base: VeltranoBaseType::Bool,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            },
            LiteralExpr::String(_) => VeltranoType {
                base: VeltranoBaseType::String,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            },
            LiteralExpr::Unit => VeltranoType {
                base: VeltranoBaseType::Unit,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            },
            LiteralExpr::Null => VeltranoType {
                base: VeltranoBaseType::Unit, // For now
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            },
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
                // Both operands must be Int
                let expected_int = VeltranoType {
                    base: VeltranoBaseType::Int,
                    ownership: Ownership::Owned,
                    mutability: Mutability::Immutable,
                };

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

                Ok(expected_int)
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

                Ok(VeltranoType {
                    base: VeltranoBaseType::Bool,
                    ownership: Ownership::Owned,
                    mutability: Mutability::Immutable,
                })
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
                // Must be Int
                let expected_int = VeltranoType {
                    base: VeltranoBaseType::Int,
                    ownership: Ownership::Owned,
                    mutability: Mutability::Immutable,
                };

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

                Ok(expected_int)
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
        Ok(VeltranoType {
            base: VeltranoBaseType::Unit,
            ownership: Ownership::Owned,
            mutability: Mutability::Immutable,
        })
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

    /// Check .ref() method call
    fn check_ref_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        match &receiver_type.ownership {
            Ownership::Owned => {
                // Own<T> → T
                Ok(VeltranoType {
                    base: receiver_type.base.clone(),
                    ownership: Ownership::Borrowed,
                    mutability: receiver_type.mutability.clone(),
                })
            }
            Ownership::Borrowed => {
                // Handle String → Str conversion
                match &receiver_type.base {
                    VeltranoBaseType::String => Ok(VeltranoType {
                        base: VeltranoBaseType::Str,
                        ownership: Ownership::Borrowed,
                        mutability: receiver_type.mutability.clone(),
                    }),
                    _ => Err(TypeCheckError::MethodNotFound {
                        receiver_type: receiver_type.clone(),
                        method: "ref".to_string(),
                        location: SourceLocation {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            source_line: "".to_string(),
                        },
                    }),
                }
            }
            Ownership::MutBorrowed => {
                // MutRef<T> → T
                Ok(VeltranoType {
                    base: receiver_type.base.clone(),
                    ownership: Ownership::Borrowed,
                    mutability: Mutability::Immutable,
                })
            }
        }
    }

    /// Check .mutRef() method call
    fn check_mut_ref_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        match &receiver_type.ownership {
            Ownership::Owned => {
                // Own<T> → MutRef<T>
                Ok(VeltranoType {
                    base: receiver_type.base.clone(),
                    ownership: Ownership::MutBorrowed,
                    mutability: Mutability::Mutable,
                })
            }
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

    /// Check .toSlice() method call
    fn check_to_slice_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        match &receiver_type.base {
            VeltranoBaseType::Vec(inner_type) => {
                // Vec<T> → Slice<T>
                Ok(VeltranoType {
                    base: VeltranoBaseType::Slice(inner_type.clone()),
                    ownership: Ownership::Borrowed,
                    mutability: receiver_type.mutability.clone(),
                })
            }
            VeltranoBaseType::Array(inner_type, _) => {
                // [T; N] → Slice<T>
                Ok(VeltranoType {
                    base: VeltranoBaseType::Slice(inner_type.clone()),
                    ownership: Ownership::Borrowed,
                    mutability: receiver_type.mutability.clone(),
                })
            }
            _ => Err(TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: "toSlice".to_string(),
                location: SourceLocation {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                    source_line: "".to_string(),
                },
            }),
        }
    }

    /// Check .clone() method call
    fn check_clone_method(
        &mut self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Check if the type implements Clone trait (hardcoded knowledge for now)
        let can_clone = match &receiver_type.base {
            VeltranoBaseType::Int | VeltranoBaseType::Bool | VeltranoBaseType::Unit => true,
            VeltranoBaseType::String => true,
            VeltranoBaseType::Str => false, // &str is Copy, not Clone
            VeltranoBaseType::Custom(_) => true, // Assume custom types can be cloned for now
            VeltranoBaseType::Vec(_)
            | VeltranoBaseType::Slice(_)
            | VeltranoBaseType::Array(_, _) => true,
        };

        if can_clone {
            // Clone returns an owned version of the same base type
            Ok(VeltranoType {
                base: receiver_type.base.clone(),
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            })
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

    /// Convert VeltranoType to Rust type name for trait checking
    fn veltrano_type_to_rust_type_name(&self, veltrano_type: &VeltranoType) -> String {
        match &veltrano_type.base {
            VeltranoBaseType::Int => "i32".to_string(),
            VeltranoBaseType::Bool => "bool".to_string(),
            VeltranoBaseType::Str => "&str".to_string(),
            VeltranoBaseType::String => "String".to_string(),
            VeltranoBaseType::Unit => "()".to_string(),
            VeltranoBaseType::Vec(element_type) => {
                format!(
                    "Vec<{}>",
                    self.veltrano_type_to_rust_type_name(element_type)
                )
            }
            VeltranoBaseType::Slice(element_type) => {
                format!("&[{}]", self.veltrano_type_to_rust_type_name(element_type))
            }
            VeltranoBaseType::Array(element_type, size) => {
                format!(
                    "[{}; {}]",
                    self.veltrano_type_to_rust_type_name(element_type),
                    size
                )
            }
            VeltranoBaseType::Custom(name) => name.clone(),
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

    /// Check .toString() method call
    fn check_tostring_method(
        &mut self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        // Check if the type implements ToString trait (hardcoded knowledge for now)
        let can_tostring = match &receiver_type.base {
            VeltranoBaseType::Int | VeltranoBaseType::Bool | VeltranoBaseType::Unit => true,
            VeltranoBaseType::String | VeltranoBaseType::Str => true,
            VeltranoBaseType::Custom(_) => true, // Assume custom types can be converted to string
            VeltranoBaseType::Vec(_)
            | VeltranoBaseType::Slice(_)
            | VeltranoBaseType::Array(_, _) => true,
        };

        if can_tostring {
            // toString returns an owned String
            Ok(VeltranoType {
                base: VeltranoBaseType::String,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            })
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

    /// Check .length() method call
    fn check_length_method(
        &self,
        receiver_type: &VeltranoType,
    ) -> Result<VeltranoType, TypeCheckError> {
        match &receiver_type.base {
            VeltranoBaseType::String | VeltranoBaseType::Str => Ok(VeltranoType {
                base: VeltranoBaseType::Int,
                ownership: Ownership::Owned,
                mutability: Mutability::Immutable,
            }),
            _ => {
                // Method not found
                Err(TypeCheckError::MethodNotFound {
                    receiver_type: receiver_type.clone(),
                    method: "length".to_string(),
                    location: SourceLocation {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                        source_line: "".to_string(),
                    },
                })
            }
        }
    }

    /// Core type equality check - no implicit conversion logic
    fn types_equal(&self, a: &VeltranoType, b: &VeltranoType) -> bool {
        a == b // Simple structural equality
    }

    /// Convert AST Type to VeltranoType
    fn convert_ast_type_to_veltrano_type(&self, ast_type: &Type) -> VeltranoType {
        let ownership = match ast_type.reference_depth {
            0 => Ownership::Owned,
            _ => Ownership::Borrowed,
        };

        let base = match &ast_type.base {
            BaseType::Int => VeltranoBaseType::Int,
            BaseType::Bool => VeltranoBaseType::Bool,
            BaseType::Unit => VeltranoBaseType::Unit,
            BaseType::Nothing => VeltranoBaseType::Unit, // Map Nothing to Unit for now
            BaseType::Str => VeltranoBaseType::Str,
            BaseType::String => VeltranoBaseType::String,
            BaseType::Custom(name) => VeltranoBaseType::Custom(name.clone()),
            BaseType::MutRef(inner) => {
                // For MutRef, convert the inner type and mark as MutBorrowed
                let inner_veltrano = self.convert_ast_type_to_veltrano_type(inner);
                return VeltranoType {
                    base: inner_veltrano.base,
                    ownership: Ownership::MutBorrowed,
                    mutability: Mutability::Mutable,
                };
            }
            BaseType::Box(inner) => {
                // For Box, treat as owned version of the inner type
                let inner_veltrano = self.convert_ast_type_to_veltrano_type(inner);
                return VeltranoType {
                    base: inner_veltrano.base,
                    ownership: Ownership::Owned,
                    mutability: inner_veltrano.mutability,
                };
            }
        };

        VeltranoType {
            base,
            ownership,
            mutability: Mutability::Immutable,
        }
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
                // Try to suggest a fix using simple pattern matching
                if let Some(suggestion) = self.suggest_conversion(&actual, &expected) {
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
                // Check if method exists on a converted version of the type
                if let Some(suggestion) = self.suggest_method_fix(&receiver_type, &method) {
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
                // Check if field exists on a converted version of the type
                if let Some(suggestion) = self.suggest_field_fix(&object_type, &field) {
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
            _ => error,
        }
    }

    fn suggest_conversion(&self, from: &VeltranoType, to: &VeltranoType) -> Option<String> {
        // Simple constant-time pattern matching for common conversions
        match (from, to) {
            // Own<T> → T conversions
            (
                VeltranoType {
                    base,
                    ownership: Ownership::Owned,
                    ..
                },
                VeltranoType {
                    base: target_base,
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) if base == target_base => Some(".ref()".to_string()),

            // Own<String> → Str conversion (double deref)
            (
                VeltranoType {
                    base: VeltranoBaseType::String,
                    ownership: Ownership::Owned,
                    ..
                },
                VeltranoType {
                    base: VeltranoBaseType::Str,
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) => Some(".ref().ref()".to_string()),

            // String → Str conversion
            (
                VeltranoType {
                    base: VeltranoBaseType::String,
                    ownership: Ownership::Borrowed,
                    ..
                },
                VeltranoType {
                    base: VeltranoBaseType::Str,
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) => Some(".ref()".to_string()),

            // Vec<T> → Slice<T> conversion
            (
                VeltranoType {
                    base: VeltranoBaseType::Vec(_),
                    ownership: Ownership::Borrowed,
                    ..
                },
                VeltranoType {
                    base: VeltranoBaseType::Slice(_),
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) => Some(".toSlice()".to_string()),

            // MutRef<T> → T conversions
            (
                VeltranoType {
                    base,
                    ownership: Ownership::MutBorrowed,
                    ..
                },
                VeltranoType {
                    base: target_base,
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) if base == target_base => Some(".ref()".to_string()),

            // Array to slice conversions: [T; N] → Slice<T>
            (
                VeltranoType {
                    base: VeltranoBaseType::Array(_, _),
                    ownership: Ownership::Borrowed,
                    ..
                },
                VeltranoType {
                    base: VeltranoBaseType::Slice(_),
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) => Some(".toSlice()".to_string()),

            // Own<[T; N]> → Slice<T> conversion (needs ref first)
            (
                VeltranoType {
                    base: VeltranoBaseType::Array(_, _),
                    ownership: Ownership::Owned,
                    ..
                },
                VeltranoType {
                    base: VeltranoBaseType::Slice(_),
                    ownership: Ownership::Borrowed,
                    ..
                },
            ) => Some(".ref().toSlice()".to_string()),

            _ => None,
        }
    }

    fn suggest_method_fix(&self, receiver_type: &VeltranoType, method: &str) -> Option<String> {
        // Check if method would be available after .ref()
        match receiver_type.ownership {
            Ownership::Owned => Some(format!(
                "Try calling .ref().{method}() if the method exists on the borrowed type"
            )),
            _ => None,
        }
    }

    fn suggest_field_fix(&self, object_type: &VeltranoType, field: &str) -> Option<String> {
        // Check if field would be available after .ref()
        match object_type.ownership {
            Ownership::Owned => Some(format!(
                "Try accessing .ref().{field} if the field exists on the borrowed type"
            )),
            _ => None,
        }
    }
}
