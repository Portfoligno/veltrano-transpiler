//! Parser for the Veltrano language
//!
//! This module implements a recursive descent parser that converts a stream of tokens
//! into an Abstract Syntax Tree (AST). It handles all language constructs including
//! expressions, statements, function declarations, and data classes.
//!
//! The parser is organized into sub-modules:
//! - `error` - Error handling and recovery
//! - `expressions` - Expression parsing
//! - `statements` - Statement parsing
//! - `types` - Type parsing
//! - `utils` - Utility functions

mod error;
mod expressions;
mod statements;
mod types;
mod utils;

use crate::ast::*;
use crate::error::{ErrorCollection, VeltranoError};
use crate::lexer::{Token, TokenType};

pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    pub(super) in_function_body: bool, // Track if we're parsing inside a function body
    pub(super) next_call_id: usize, // Counter for unique call IDs (both method and function calls)
    errors: ErrorCollection,        // Collection of errors encountered during parsing
    panic_mode: bool,               // Flag to avoid cascading errors after a syntax error
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            in_function_body: false,
            next_call_id: 0,
            errors: ErrorCollection::new(),
            panic_mode: false,
        }
    }

    pub fn parse(&mut self) -> Result<Program, VeltranoError> {
        let (program, errors) = self.parse_with_recovery();
        // Return the first error for backward compatibility
        match errors.errors().first() {
            Some(error) => Err(error.clone()),
            None => Ok(program),
        }
    }

    /// Parse with error recovery, returning both the program and any errors found
    pub fn parse_with_recovery(&mut self) -> (Program, ErrorCollection) {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            // Handle comment tokens
            if let Some(comment_stmt) = self.try_parse_comment() {
                statements.push(comment_stmt);
                continue;
            }

            // Exit panic mode on successful parse
            self.panic_mode = false;

            match self.declaration() {
                Ok(stmts) => statements.extend(stmts.into_iter()),
                Err(err) => {
                    // Record the error
                    self.errors.add_error(err);

                    // Enter panic mode to avoid cascading errors
                    self.panic_mode = true;

                    // Try to recover by synchronizing to a safe point
                    self.synchronize();
                }
            }
        }

        // Second pass: analyze bump usage and update has_hidden_bump flags
        Self::analyze_bump_usage(&mut statements);

        let errors = std::mem::replace(&mut self.errors, ErrorCollection::new());
        (Program { statements }, errors)
    }

    /// Analyzes bump usage across all functions and updates has_hidden_bump flags
    fn analyze_bump_usage(statements: &mut Vec<Stmt>) {
        use std::collections::HashSet;

        // Keep iterating until no changes are made (to handle transitive dependencies)
        let mut changed = true;
        let mut functions_with_bump = HashSet::new();

        while changed {
            changed = false;

            for stmt in statements.iter_mut() {
                if let Stmt::FunDecl(fun_decl) = stmt {
                    let old_value = fun_decl.has_hidden_bump;
                    let should_have_bump = fun_decl.needs_lifetime_params(&functions_with_bump);

                    if should_have_bump != old_value {
                        fun_decl.has_hidden_bump = should_have_bump;
                        changed = true;
                    }

                    // Add to functions_with_bump if it has bump parameter (for transitive dependencies)
                    if should_have_bump && !functions_with_bump.contains(&fun_decl.name) {
                        functions_with_bump.insert(fun_decl.name.clone());
                        changed = true;
                    } else if !should_have_bump && functions_with_bump.contains(&fun_decl.name) {
                        functions_with_bump.remove(&fun_decl.name);
                        changed = true;
                    }
                }
            }
        }
    }
}
