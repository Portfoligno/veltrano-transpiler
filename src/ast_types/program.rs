//! Program structure definition
//!
//! This module contains the top-level Program type that represents
//! a complete Veltrano source file.

use super::Stmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
