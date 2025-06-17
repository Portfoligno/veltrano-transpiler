//! Parser for the Veltrano language
//!
//! This module implements a recursive descent parser that converts a stream of tokens
//! into an Abstract Syntax Tree (AST). It handles all language constructs including
//! expressions, statements, function declarations, and data classes.

mod error;
mod expressions;
mod statements;
mod utils;

use crate::ast::*;
use crate::ast_types::Located;
use crate::error::{ErrorCollection, SourceLocation, Span, VeltranoError};
use crate::lexer::{Token, TokenType};
use crate::types::VeltranoType;

pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    pub(super) in_function_body: bool,  // Track if we're parsing inside a function body
    pub(super) next_call_id: usize,     // Counter for unique call IDs (both method and function calls)
    errors: ErrorCollection, // Collection of errors encountered during parsing
    panic_mode: bool,        // Flag to avoid cascading errors after a syntax error
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


    pub(super) fn parse_type(&mut self) -> Result<Located<VeltranoType>, VeltranoError> {
        let start_token = self.peek();
        let start_location = SourceLocation::new(start_token.line, start_token.column);
        let vtype = self.parse_type_inner()?;
        let end_token = self.previous();
        let end_location = SourceLocation::new(end_token.line, end_token.column);
        Ok(Located::new(vtype, Span::new(start_location, end_location)))
    }

    fn parse_type_inner(&mut self) -> Result<VeltranoType, VeltranoError> {
        if let TokenType::Identifier(type_name) = &self.peek().token_type {
            let type_name = type_name.clone();
            self.advance();

            match type_name.as_str() {
                // Signed integers
                "I32" => Ok(VeltranoType::i32()),
                "I64" => Ok(VeltranoType::i64()),
                "ISize" => Ok(VeltranoType::isize()),
                // Unsigned integers
                "U32" => Ok(VeltranoType::u32()),
                "U64" => Ok(VeltranoType::u64()),
                "USize" => Ok(VeltranoType::usize()),
                // Other primitives
                "Bool" => Ok(VeltranoType::bool()),
                "Char" => Ok(VeltranoType::char()),
                "Unit" => Ok(VeltranoType::unit()),
                "Nothing" => Ok(VeltranoType::nothing()),
                // String types
                "Str" => Ok(VeltranoType::str()), // naturally referenced
                "String" => Ok(VeltranoType::string()), // naturally referenced
                "Ref" => self.parse_ref_type(),
                "Own" => self.parse_own_type(),
                "MutRef" => self.parse_mutref_type(),
                "Box" => self.parse_box_type(),
                "Vec" => self.parse_vec_type(),
                "Array" => self.parse_array_type(),
                "Option" => self.parse_option_type(),
                "Result" => self.parse_result_type(),
                _ => Ok(VeltranoType::custom(type_name)), // naturally referenced
            }
        } else {
            Err(self.syntax_error("Expected type name".to_string()))
        }
    }

    fn parse_ref_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Ref")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::ref_(inner_type.node))
    }

    fn parse_own_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Own")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;

        // Validation is now handled by the type checker
        Ok(VeltranoType::own(inner_type.node))
    }

    fn parse_mutref_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after MutRef")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::mut_ref(inner_type.node))
    }

    fn parse_box_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Box")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::boxed(inner_type.node))
    }

    fn parse_vec_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Vec")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::vec(inner_type.node))
    }

    fn parse_array_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Array")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Comma, "Expected ',' after array element type")?;

        // Parse array size
        if let TokenType::IntLiteral(size) = &self.peek().token_type {
            let size = *size as usize;
            self.advance();
            self.consume(&TokenType::Greater, "Expected '>' after array size")?;
            Ok(VeltranoType::array(inner_type.node, size))
        } else {
            Err(self.syntax_error("Expected integer literal for array size".to_string()))
        }
    }

    fn parse_option_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Option")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::option(inner_type.node))
    }

    fn parse_result_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Result")?;
        let ok_type = self.parse_type()?;
        self.consume(&TokenType::Comma, "Expected ',' after Result ok type")?;
        let err_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after Result error type")?;
        Ok(VeltranoType::result(ok_type.node, err_type.node))
    }
}

