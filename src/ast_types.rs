//! Abstract Syntax Tree (AST) type definitions for Veltrano
//!
//! This module contains all the AST node types used by the Veltrano transpiler,
//! including expressions, statements, and the program structure. It also provides
//! extension traits for AST traversal and analysis.

use crate::error::Span;
use crate::types::VeltranoType;

/// A wrapper for AST nodes that includes source location information
#[derive(Debug, Clone)]
pub struct Located<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Located<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

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
pub enum Argument {
    Bare(LocatedExpr, Option<(String, String)>), // Expression with optional inline comment
    Named(String, LocatedExpr, Option<(String, String)>), // Named argument with optional inline comment
    Shorthand(String, Option<(String, String)>), // Shorthand field (.field) with optional inline comment
    StandaloneComment(String, String), // Standalone comment with content and preceding whitespace
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

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(LocatedExpr),
    VarDecl(VarDeclStmt),
    FunDecl(FunDeclStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(Option<LocatedExpr>),
    Block(Vec<Stmt>),
    Comment(CommentStmt),     // All comments (standalone and inline)
    Import(ImportStmt),       // Import statement
    DataClass(DataClassStmt), // Data class declaration
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentContext {
    OwnLine,   // Comment on its own line
    EndOfLine, // Comment at the end of a line with code
}

#[derive(Debug, Clone)]
pub struct CommentStmt {
    pub content: String,
    pub is_block_comment: bool,
    pub preceding_whitespace: String,
    pub context: CommentContext,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub name: String,
    pub type_annotation: Option<Located<VeltranoType>>,
    pub initializer: Option<LocatedExpr>,
}

#[derive(Debug, Clone)]
pub struct FunDeclStmt {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Located<VeltranoType>>,
    pub body: Box<Stmt>,
    pub has_hidden_bump: bool, // Whether this function should receive a hidden bump parameter
}

impl FunDeclStmt {
    /// Analyzes if this function actually uses bump allocation (not just reference types)
    pub fn uses_bump_allocation(
        &self,
        functions_with_bump: &std::collections::HashSet<String>,
    ) -> bool {
        use crate::ast::query::AstQuery;

        if self.name == "main" {
            return false;
        }

        // Check if the function body uses bump allocation
        if AstQuery::stmt_uses_bump_allocation(&self.body) {
            return true;
        }

        // Check if we call any functions that use bump
        let mut uses_bump = false;
        let _ = self.body.walk_expressions(&mut |expr| {
            if let Expr::Call(call) = &expr.node {
                if let Expr::Identifier(name) = &call.callee.node {
                    if functions_with_bump.contains(name) {
                        uses_bump = true;
                        return Err(()); // Early exit
                    }
                }
            }
            Ok::<(), ()>(())
        });

        uses_bump
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
            if self.type_needs_lifetime(&param.param_type.node) {
                return true;
            }
        }

        // Check return type for reference types
        if let Some(return_type) = &self.return_type {
            if self.type_needs_lifetime(&return_type.node) {
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

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Located<VeltranoType>,
    pub inline_comment: Option<(String, String)>, // Optional inline comment after parameter
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: LocatedExpr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: LocatedExpr,
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
    pub field_type: Located<VeltranoType>,
    pub inline_comment: Option<(String, String)>, // Inline comment after field
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// Extension trait for expression traversal
///
/// Provides functional-style traversal methods for AST analysis with early exit.
pub trait ExprExt {
    /// Walk the expression tree in pre-order
    ///
    /// Calls visitor on current node before its children. Return Err to stop early.
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>;

    /// Walk the expression tree in post-order
    ///
    /// Calls visitor on children before current node. Useful for bottom-up analysis.
    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>;

    /// Find all sub-expressions matching a predicate
    ///
    /// Returns vector of all expressions matching the predicate.
    #[allow(dead_code)]
    fn find_subexpressions<F>(&self, predicate: F) -> Vec<&LocatedExpr>
    where
        F: Fn(&LocatedExpr) -> bool;

    /// Check if any sub-expression matches a predicate
    ///
    /// Short-circuits on first match for efficiency.
    #[allow(dead_code)]
    fn any_subexpr<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool;

    /// Check if all sub-expressions match a predicate
    ///
    /// Short-circuits on first non-match for efficiency.
    #[allow(dead_code)]
    fn all_subexprs<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool;
}

impl ExprExt for LocatedExpr {
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>,
    {
        // Visit this node first (pre-order)
        visitor(self)?;

        // Then visit children
        match &self.node {
            Expr::Binary(binary) => {
                binary.left.walk(visitor)?;
                binary.right.walk(visitor)?;
            }
            Expr::Unary(unary) => {
                unary.operand.walk(visitor)?;
            }
            Expr::Call(call) => {
                call.callee.walk(visitor)?;
                for arg in &call.args {
                    match arg {
                        Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                            expr.walk(visitor)?;
                        }
                        _ => {}
                    }
                }
            }
            Expr::MethodCall(method_call) => {
                method_call.object.walk(visitor)?;
                for arg in &method_call.args {
                    arg.walk(visitor)?;
                }
            }
            Expr::FieldAccess(field_access) => {
                field_access.object.walk(visitor)?;
            }
            Expr::Literal(_) | Expr::Identifier(_) => {
                // Leaf nodes - no children to visit
            }
        }
        Ok(())
    }

    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>,
    {
        // Visit children first (post-order)
        match &self.node {
            Expr::Binary(binary) => {
                binary.left.walk_post(visitor)?;
                binary.right.walk_post(visitor)?;
            }
            Expr::Unary(unary) => {
                unary.operand.walk_post(visitor)?;
            }
            Expr::Call(call) => {
                call.callee.walk_post(visitor)?;
                for arg in &call.args {
                    match arg {
                        Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                            expr.walk_post(visitor)?;
                        }
                        _ => {}
                    }
                }
            }
            Expr::MethodCall(method_call) => {
                method_call.object.walk_post(visitor)?;
                for arg in &method_call.args {
                    arg.walk_post(visitor)?;
                }
            }
            Expr::FieldAccess(field_access) => {
                field_access.object.walk_post(visitor)?;
            }
            Expr::Literal(_) | Expr::Identifier(_) => {
                // Leaf nodes - no children to visit
            }
        }

        // Then visit this node
        visitor(self)
    }

    fn find_subexpressions<F>(&self, predicate: F) -> Vec<&LocatedExpr>
    where
        F: Fn(&LocatedExpr) -> bool,
    {
        fn collect<'a, F>(expr: &'a LocatedExpr, predicate: &F, results: &mut Vec<&'a LocatedExpr>)
        where
            F: Fn(&LocatedExpr) -> bool,
        {
            if predicate(expr) {
                results.push(expr);
            }

            match &expr.node {
                Expr::Binary(binary) => {
                    collect(&binary.left, predicate, results);
                    collect(&binary.right, predicate, results);
                }
                Expr::Unary(unary) => {
                    collect(&unary.operand, predicate, results);
                }
                Expr::Call(call) => {
                    collect(&call.callee, predicate, results);
                    for arg in &call.args {
                        match arg {
                            Argument::Bare(e, _) | Argument::Named(_, e, _) => {
                                collect(e, predicate, results);
                            }
                            _ => {}
                        }
                    }
                }
                Expr::MethodCall(method_call) => {
                    collect(&method_call.object, predicate, results);
                    for arg in &method_call.args {
                        collect(arg, predicate, results);
                    }
                }
                Expr::FieldAccess(field_access) => {
                    collect(&field_access.object, predicate, results);
                }
                Expr::Literal(_) | Expr::Identifier(_) => {}
            }
        }

        let mut results = Vec::new();
        collect(self, &predicate, &mut results);
        results
    }

    fn any_subexpr<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool,
    {
        let mut found = false;
        let _ = self.walk(&mut |expr| {
            if predicate(expr) {
                found = true;
                Err(()) // Early exit
            } else {
                Ok(())
            }
        });
        found
    }

    fn all_subexprs<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool,
    {
        let mut all_match = true;
        let _ = self.walk(&mut |expr| {
            if !predicate(expr) {
                all_match = false;
                Err(()) // Early exit
            } else {
                Ok(())
            }
        });
        all_match
    }
}

/// Extension trait for statement traversal
///
/// Provides traversal methods for analyzing control flow, declarations, and nested structures.
pub trait StmtExt {
    /// Walk the statement tree in pre-order
    ///
    /// Calls visitor on each statement before its children. Return Err to stop early.
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>;

    /// Walk the statement tree in post-order
    ///
    /// Useful for bottom-up analysis or cleanup operations.
    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>;

    /// Find all sub-statements matching a predicate
    ///
    /// Recursively searches the statement tree for matching statements.
    #[allow(dead_code)]
    fn find_statements<F>(&self, predicate: F) -> Vec<&Stmt>
    where
        F: Fn(&Stmt) -> bool;

    /// Walk all expressions within this statement
    ///
    /// Visits all expressions in the statement tree for expression-level analysis.
    fn walk_expressions<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>;

    /// Check if the statement tree can exit early
    ///
    /// Returns true if the statement or any sub-statement contains a return.
    fn can_exit_early(&self) -> bool;
}

impl StmtExt for Stmt {
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>,
    {
        // Visit this node first (pre-order)
        visitor(self)?;

        // Then visit children
        match self {
            Stmt::Block(statements) => {
                for stmt in statements {
                    stmt.walk(visitor)?;
                }
            }
            Stmt::If(if_stmt) => {
                if_stmt.then_branch.walk(visitor)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.walk(visitor)?;
                }
            }
            Stmt::While(while_stmt) => {
                while_stmt.body.walk(visitor)?;
            }
            Stmt::FunDecl(fun_decl) => {
                fun_decl.body.walk(visitor)?;
            }
            // Leaf nodes
            Stmt::Expression(_)
            | Stmt::VarDecl(_)
            | Stmt::Return(_)
            | Stmt::Comment(_)
            | Stmt::Import(_)
            | Stmt::DataClass(_) => {}
        }
        Ok(())
    }

    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>,
    {
        // Visit children first (post-order)
        match self {
            Stmt::Block(statements) => {
                for stmt in statements {
                    stmt.walk_post(visitor)?;
                }
            }
            Stmt::If(if_stmt) => {
                if_stmt.then_branch.walk_post(visitor)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.walk_post(visitor)?;
                }
            }
            Stmt::While(while_stmt) => {
                while_stmt.body.walk_post(visitor)?;
            }
            Stmt::FunDecl(fun_decl) => {
                fun_decl.body.walk_post(visitor)?;
            }
            // Leaf nodes
            Stmt::Expression(_)
            | Stmt::VarDecl(_)
            | Stmt::Return(_)
            | Stmt::Comment(_)
            | Stmt::Import(_)
            | Stmt::DataClass(_) => {}
        }

        // Then visit this node
        visitor(self)
    }

    fn find_statements<F>(&self, predicate: F) -> Vec<&Stmt>
    where
        F: Fn(&Stmt) -> bool,
    {
        fn collect<'a, F>(stmt: &'a Stmt, predicate: &F, results: &mut Vec<&'a Stmt>)
        where
            F: Fn(&Stmt) -> bool,
        {
            if predicate(stmt) {
                results.push(stmt);
            }

            match stmt {
                Stmt::Block(statements) => {
                    for s in statements {
                        collect(s, predicate, results);
                    }
                }
                Stmt::If(if_stmt) => {
                    collect(&if_stmt.then_branch, predicate, results);
                    if let Some(else_branch) = &if_stmt.else_branch {
                        collect(else_branch, predicate, results);
                    }
                }
                Stmt::While(while_stmt) => {
                    collect(&while_stmt.body, predicate, results);
                }
                Stmt::FunDecl(fun_decl) => {
                    collect(&fun_decl.body, predicate, results);
                }
                _ => {}
            }
        }

        let mut results = Vec::new();
        collect(self, &predicate, &mut results);
        results
    }

    fn walk_expressions<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>,
    {
        match self {
            Stmt::Expression(expr) => expr.walk(visitor)?,
            Stmt::VarDecl(var_decl) => {
                if let Some(init) = &var_decl.initializer {
                    init.walk(visitor)?;
                }
            }
            Stmt::Return(Some(expr)) => expr.walk(visitor)?,
            Stmt::If(if_stmt) => {
                if_stmt.condition.walk(visitor)?;
                if_stmt.then_branch.walk_expressions(visitor)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.walk_expressions(visitor)?;
                }
            }
            Stmt::While(while_stmt) => {
                while_stmt.condition.walk(visitor)?;
                while_stmt.body.walk_expressions(visitor)?;
            }
            Stmt::Block(statements) => {
                for stmt in statements {
                    stmt.walk_expressions(visitor)?;
                }
            }
            Stmt::FunDecl(fun_decl) => {
                fun_decl.body.walk_expressions(visitor)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn can_exit_early(&self) -> bool {
        match self {
            Stmt::Return(_) => true,
            Stmt::Block(statements) => statements.iter().any(|s| s.can_exit_early()),
            Stmt::If(if_stmt) => {
                if_stmt.then_branch.can_exit_early()
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |s| s.can_exit_early())
            }
            Stmt::While(while_stmt) => while_stmt.body.can_exit_early(),
            Stmt::FunDecl(fun_decl) => fun_decl.body.can_exit_early(),
            _ => false,
        }
    }
}
