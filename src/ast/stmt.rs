//! Statement AST node definitions
//!
//! This module contains all statement-related AST types including
//! declarations, control flow, and import statements.

use super::{Expr, Located, LocatedExpr, StmtExt};
use crate::types::VeltranoType;

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
    pub location: crate::error::SourceLocation,
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
