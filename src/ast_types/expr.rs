//! Expression AST node definitions
//!
//! This module contains all expression-related AST types including
//! literals, operators, and various expression forms.

use super::Located;

/// Type alias for located expressions
pub type LocatedExpr = Located<Expr>;

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
    pub operand: Box<LocatedExpr>,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Minus,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<LocatedExpr>,
    pub comment_after_left: Option<(String, String)>, // Optional comment after left operand
    pub operator: BinaryOp,
    pub comment_after_operator: Option<(String, String)>, // Optional comment after operator
    pub right: Box<LocatedExpr>,
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
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct ArgumentComment {
    pub before: Option<(String, String)>,
    pub after: Option<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum Argument {
    Bare(LocatedExpr, ArgumentComment), // Expression with comments
    Named(String, LocatedExpr, ArgumentComment), // Named argument with comments
    Shorthand(String, ArgumentComment), // Shorthand field (.field) with comments
    StandaloneComment(String, String),  // Standalone comment with content and preceding whitespace
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<LocatedExpr>,
    pub args: Vec<Argument>,
    pub is_multiline: bool, // Whether this call was originally formatted across multiple lines
    pub id: usize,          // Unique ID for type resolution tracking
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Box<LocatedExpr>,
    pub method: String,
    pub args: Vec<LocatedExpr>,
    pub inline_comment: Option<(String, String)>, // Optional inline comment after method call
    pub id: usize,                                // Unique ID for type resolution tracking
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<LocatedExpr>,
    pub field: String,
}
