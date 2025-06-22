//! Abstract Syntax Tree (AST) type definitions for Veltrano
//!
//! This module contains all the AST node types used by the Veltrano transpiler,
//! including expressions, statements, and the program structure. It also provides
//! extension traits for AST traversal and analysis.

mod expr;
mod program;
mod stmt;
mod traversal;

use crate::error::Span;

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

// Re-export all submodule types
pub use expr::*;
pub use program::*;
pub use stmt::*;
pub use traversal::*;

// Declare other submodules
pub mod query;
