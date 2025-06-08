use crate::types::VeltranoType;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Identifier(String),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    FieldAccess(FieldAccessExpr),
}

#[derive(Debug, Clone)]
pub enum LiteralExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Unit,
    Null,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Minus,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: BinaryOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum Argument {
    Bare(Expr, Option<(String, String)>), // Expression with optional inline comment
    Named(String, Expr, Option<(String, String)>), // Named argument with optional inline comment
    Shorthand(String, Option<(String, String)>), // Shorthand field (.field) with optional inline comment
    StandaloneComment(String, String), // Standalone comment with content and preceding whitespace
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Argument>,
    pub is_multiline: bool, // Whether this call was originally formatted across multiple lines
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Box<Expr>,
    pub method: String,
    pub args: Vec<Expr>,
    pub inline_comment: Option<(String, String)>, // Optional inline comment after method call
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<Expr>,
    pub field: String,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr, Option<(String, String)>), // Expression with optional inline comment (content, whitespace)
    VarDecl(VarDeclStmt, Option<(String, String)>), // Variable declaration with optional inline comment (content, whitespace)
    FunDecl(FunDeclStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(Option<Expr>, Option<(String, String)>), // Return statement with optional inline comment (content, whitespace)
    Block(Vec<Stmt>),
    Comment(CommentStmt),     // Standalone comments
    Import(ImportStmt),       // Import statement
    DataClass(DataClassStmt), // Data class declaration
}

#[derive(Debug, Clone)]
pub struct CommentStmt {
    pub content: String,
    pub is_block_comment: bool,
    pub preceding_whitespace: String,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub name: String,
    pub type_annotation: Option<VeltranoType>,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunDeclStmt {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<VeltranoType>,
    pub body: Box<Stmt>,
    pub has_hidden_bump: bool, // Whether this function should receive a hidden bump parameter
}

impl FunDeclStmt {
    /// Analyzes if this function actually uses bump allocation (not just reference types)
    pub fn uses_bump_allocation(
        &self,
        functions_with_bump: &std::collections::HashSet<String>,
    ) -> bool {
        self.name != "main" && bump_usage_analyzer::stmt_uses_bump(&self.body, functions_with_bump)
    }

    /// Analyzes if this function needs lifetime parameters (for bump allocation or reference handling)
    pub fn needs_lifetime_params(
        &self,
        functions_with_bump: &std::collections::HashSet<String>,
    ) -> bool {
        if self.name == "main" {
            return false;
        }

        // Check if function uses bump allocation
        if self.uses_bump_allocation(functions_with_bump) {
            return true;
        }

        // Check if function has reference types in parameters or return type
        if self.has_reference_types() {
            return true;
        }

        false
    }

    /// Checks if this function has reference types in its signature
    fn has_reference_types(&self) -> bool {
        // Check parameters for reference types
        for param in &self.params {
            if self.type_needs_lifetime(&param.param_type) {
                return true;
            }
        }

        // Check return type for reference types
        if let Some(return_type) = &self.return_type {
            if self.type_needs_lifetime(return_type) {
                return true;
            }
        }

        false
    }

    /// Checks if a type needs lifetime parameters
    fn type_needs_lifetime(&self, type_: &VeltranoType) -> bool {
        use crate::types::TypeConstructor;

        match &type_.constructor {
            TypeConstructor::Str | TypeConstructor::String => true,
            TypeConstructor::Custom(_) => true, // Custom types might have lifetimes
            TypeConstructor::MutRef | TypeConstructor::Ref => {
                // Check the inner type if it has args
                if let Some(inner) = type_.inner() {
                    self.type_needs_lifetime(inner)
                } else {
                    false
                }
            }
            TypeConstructor::Box => {
                // Check the inner type if it has args
                if let Some(inner) = type_.inner() {
                    self.type_needs_lifetime(inner)
                } else {
                    false
                }
            }
            TypeConstructor::I32
            | TypeConstructor::I64
            | TypeConstructor::ISize
            | TypeConstructor::U32
            | TypeConstructor::U64
            | TypeConstructor::USize
            | TypeConstructor::Bool
            | TypeConstructor::Char
            | TypeConstructor::Unit
            | TypeConstructor::Nothing => false,
            // For other constructors, conservatively assume they might need lifetimes
            _ => type_.args.iter().any(|arg| self.type_needs_lifetime(arg)),
        }
    }
}

mod bump_usage_analyzer {
    use super::*;
    use std::collections::HashSet;

    pub fn stmt_uses_bump(stmt: &Stmt, functions_with_bump: &HashSet<String>) -> bool {
        match stmt {
            Stmt::Expression(expr, _) => expr_uses_bump(expr, functions_with_bump),
            Stmt::VarDecl(var_decl, _) => {
                if let Some(initializer) = &var_decl.initializer {
                    expr_uses_bump(initializer, functions_with_bump)
                } else {
                    false
                }
            }
            Stmt::FunDecl(_) => false, // Nested function declarations don't affect bump usage
            Stmt::If(if_stmt) => {
                expr_uses_bump(&if_stmt.condition, functions_with_bump)
                    || stmt_uses_bump(&if_stmt.then_branch, functions_with_bump)
                    || if_stmt.else_branch.as_ref().map_or(false, |else_branch| {
                        stmt_uses_bump(else_branch, functions_with_bump)
                    })
            }
            Stmt::While(while_stmt) => {
                expr_uses_bump(&while_stmt.condition, functions_with_bump)
                    || stmt_uses_bump(&while_stmt.body, functions_with_bump)
            }
            Stmt::Return(expr_opt, _) => expr_opt
                .as_ref()
                .map_or(false, |expr| expr_uses_bump(expr, functions_with_bump)),
            Stmt::Block(statements) => statements
                .iter()
                .any(|stmt| stmt_uses_bump(stmt, functions_with_bump)),
            Stmt::Comment(_) | Stmt::Import(_) | Stmt::DataClass(_) => false,
        }
    }

    pub fn expr_uses_bump(expr: &Expr, functions_with_bump: &HashSet<String>) -> bool {
        match expr {
            Expr::Literal(_) | Expr::Identifier(_) => false,
            Expr::Unary(unary) => expr_uses_bump(&unary.operand, functions_with_bump),
            Expr::Binary(binary) => {
                expr_uses_bump(&binary.left, functions_with_bump)
                    || expr_uses_bump(&binary.right, functions_with_bump)
            }
            Expr::Call(call) => {
                // Check if calling a function that uses bump
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if functions_with_bump.contains(name) {
                        return true;
                    }
                }
                // Check arguments
                expr_uses_bump(&call.callee, functions_with_bump)
                    || call.args.iter().any(|arg| match arg {
                        Argument::Bare(expr, _) => expr_uses_bump(expr, functions_with_bump),
                        Argument::Named(_, expr, _) => expr_uses_bump(expr, functions_with_bump),
                        Argument::Shorthand(_, _) => false, // Shorthand is just an identifier, doesn't use bump allocation
                        Argument::StandaloneComment(_, _) => false, // Comments don't use bump allocation
                    })
            }
            Expr::MethodCall(method_call) => {
                // Check for .bumpRef() method calls
                if method_call.method == "bumpRef" {
                    return true;
                }
                // Check object and arguments
                expr_uses_bump(&method_call.object, functions_with_bump)
                    || method_call
                        .args
                        .iter()
                        .any(|expr| expr_uses_bump(expr, functions_with_bump))
            }
            Expr::FieldAccess(field_access) => {
                expr_uses_bump(&field_access.object, functions_with_bump)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: VeltranoType,
    pub inline_comment: Option<(String, String)>, // Optional inline comment after parameter
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub type_name: String,
    pub method_name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DataClassStmt {
    pub name: String,
    pub fields: Vec<DataClassField>,
}

#[derive(Debug, Clone)]
pub struct DataClassField {
    pub name: String,
    pub field_type: VeltranoType,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
