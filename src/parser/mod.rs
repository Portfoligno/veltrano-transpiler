//! Parser for the Veltrano language
//!
//! This module implements a recursive descent parser that converts a stream of tokens
//! into an Abstract Syntax Tree (AST). It handles all language constructs including
//! expressions, statements, function declarations, and data classes.

mod error;
mod expressions;
mod utils;

use crate::ast::*;
use crate::ast_types::{CommentContext, CommentStmt, Located};
use crate::error::{ErrorCollection, ErrorKind, SourceLocation, Span, VeltranoError};
use crate::lexer::{Token, TokenType};
use crate::types::VeltranoType;
use nonempty::NonEmpty;

pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    in_function_body: bool,  // Track if we're parsing inside a function body
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

    fn declaration(&mut self) -> Result<NonEmpty<Stmt>, VeltranoError> {
        if self.match_token(&TokenType::Fun) {
            Ok(NonEmpty::singleton(self.function_declaration()?))
        } else if self.match_token(&TokenType::Val) {
            self.var_declaration()
        } else if self.match_token(&TokenType::Import) {
            Ok(NonEmpty::singleton(self.import_declaration()?))
        } else if self.match_token(&TokenType::Data) {
            Ok(NonEmpty::singleton(self.data_class_declaration()?))
        } else {
            self.statement()
        }
    }

    fn function_declaration(&mut self) -> Result<Stmt, VeltranoError> {
        let name = self.consume_identifier("Expected function name")?;

        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                // Skip any newlines and comments before parsing the parameter
                self.skip_newlines_and_comments();

                let param_name = self.consume_identifier("Expected parameter name")?;

                // Enhanced error message for missing colon
                if !self.check(&TokenType::Colon) {
                    let token = self.peek();
                    let mut err = self.error(
                        ErrorKind::SyntaxError,
                        format!(
                            "Expected ':' after parameter name '{}', found {:?}",
                            param_name, token.token_type
                        ),
                    );

                    // Add helpful note
                    err = err.with_note("Function parameters must have explicit types in Veltrano");
                    err = err.with_help(format!("Try: {}:{} <type>", param_name, ""));

                    return Err(err);
                }
                self.advance(); // consume the colon

                let param_type = self.parse_type()?;

                // Capture comment immediately after the parameter type
                let inline_comment = self.skip_newlines_and_capture_comment();

                params.push(Parameter {
                    name: param_name,
                    param_type,
                    inline_comment,
                });

                if !self.match_token(&TokenType::Comma) {
                    break;
                }

                // Capture any comment after the comma for the PREVIOUS parameter
                // This handles patterns like: x: Int, // The x coordinate
                let comment_after_comma = self.skip_newlines_and_capture_comment();

                // If we found a comment after the comma, update the last parameter
                if let Some(comment) = comment_after_comma {
                    if let Some(last_param) = params.last_mut() {
                        if last_param.inline_comment.is_none() {
                            last_param.inline_comment = Some(comment);
                        }
                    }
                }
            }
        }

        // Skip any newlines and comments before the closing parenthesis
        self.skip_newlines_and_comments();

        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Skip any newlines and comments before the opening brace
        self.skip_newlines_and_comments();

        self.consume(&TokenType::LeftBrace, "Expected '{' before function body")?;

        // Set context flag before parsing function body
        let was_in_function_body = self.in_function_body;
        self.in_function_body = true;
        let body = Box::new(self.block_statement()?);
        self.in_function_body = was_in_function_body;

        Ok(Stmt::FunDecl(FunDeclStmt {
            name: name.clone(),
            params,
            return_type,
            body,
            has_hidden_bump: false, // Will be set by analyze_bump_usage
        }))
    }

    fn var_declaration(&mut self) -> Result<NonEmpty<Stmt>, VeltranoError> {
        let name = self.consume_identifier("Expected variable name")?;

        let type_annotation = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        let inline_comment = self.consume_newline()?;

        let var_decl = Stmt::VarDecl(VarDeclStmt {
            name,
            type_annotation,
            initializer,
        });

        // Add inline comment as a separate statement if present
        if let Some((content, whitespace)) = inline_comment {
            let comment = Stmt::Comment(CommentStmt {
                content,
                is_block_comment: false, // inline comments are line comments
                preceding_whitespace: whitespace,
                context: CommentContext::EndOfLine,
            });
            Ok(NonEmpty::from((var_decl, vec![comment])))
        } else {
            Ok(NonEmpty::singleton(var_decl))
        }
    }

    fn import_declaration(&mut self) -> Result<Stmt, VeltranoError> {
        // import Type.method [as alias]
        let type_name = self.consume_identifier("Expected type name after 'import'")?;
        self.consume(&TokenType::Dot, "Expected '.' after type name")?;
        let method_name = self.consume_identifier("Expected method name after '.'")?;

        let alias = if self.match_token(&TokenType::As) {
            Some(self.consume_identifier("Expected alias name after 'as'")?)
        } else {
            None
        };

        self.consume_newline()?;

        Ok(Stmt::Import(ImportStmt {
            type_name,
            method_name,
            alias,
        }))
    }

    fn data_class_declaration(&mut self) -> Result<Stmt, VeltranoError> {
        // data class ClassName(val field1: Type1, val field2: Type2, ...)
        self.consume(&TokenType::Class, "Expected 'class' after 'data'")?;
        let name = self.consume_identifier("Expected data class name after 'data class'")?;

        self.consume(&TokenType::LeftParen, "Expected '(' after data class name")?;

        let mut fields = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                // Skip any newlines and comments before parsing the field
                self.skip_newlines_and_comments();

                // Each field starts with 'val'
                self.consume(&TokenType::Val, "Expected 'val' before field name")?;
                let field_name = self.consume_identifier("Expected field name after 'val'")?;
                self.consume(&TokenType::Colon, "Expected ':' after field name")?;
                let field_type = self.parse_type()?;

                // Capture comment immediately after the field type
                let inline_comment = self.skip_newlines_and_capture_comment();

                fields.push(DataClassField {
                    name: field_name,
                    field_type,
                    inline_comment,
                });

                if !self.match_token(&TokenType::Comma) {
                    break;
                }

                // Capture any comment after the comma for the PREVIOUS field
                let comment_after_comma = self.skip_newlines_and_capture_comment();

                // If we found a comment after the comma, update the last field
                if let Some(comment) = comment_after_comma {
                    if let Some(last_field) = fields.last_mut() {
                        if last_field.inline_comment.is_none() {
                            last_field.inline_comment = Some(comment);
                        }
                    }
                }
            }
        }

        // Skip any newlines and comments before the closing parenthesis
        self.skip_newlines_and_comments();

        self.consume(
            &TokenType::RightParen,
            "Expected ')' after data class fields",
        )?;
        self.consume_newline()?;

        Ok(Stmt::DataClass(DataClassStmt { name, fields }))
    }

    fn statement(&mut self) -> Result<NonEmpty<Stmt>, VeltranoError> {
        if self.match_token(&TokenType::If) {
            Ok(NonEmpty::singleton(self.if_statement()?))
        } else if self.match_token(&TokenType::While) {
            Ok(NonEmpty::singleton(self.while_statement()?))
        } else if self.match_token(&TokenType::Return) {
            Ok(NonEmpty::singleton(self.return_statement()?))
        } else if self.match_token(&TokenType::LeftBrace) {
            Ok(NonEmpty::singleton(self.block_statement()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, VeltranoError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;

        let then_stmts = self.statement()?;
        let then_branch = Parser::nonempty_to_stmt(then_stmts);

        // Look ahead for else without consuming comments
        let mut lookahead_pos = self.current;
        let mut found_else = false;
        
        // Skip newlines and look for else
        while lookahead_pos < self.tokens.len() {
            match &self.tokens[lookahead_pos].token_type {
                TokenType::Newline => {
                    lookahead_pos += 1;
                }
                TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                    // Skip comments in lookahead but don't consume them yet
                    lookahead_pos += 1;
                }
                TokenType::Else => {
                    found_else = true;
                    break;
                }
                _ => {
                    // Not else, stop looking
                    break;
                }
            }
        }
        
        let else_branch = if found_else {
            // Now consume everything up to and including else
            while self.current < lookahead_pos {
                self.advance();
            }
            self.advance(); // consume else token
            
            let else_stmts = self.statement()?;
            Some(Parser::nonempty_to_stmt(else_stmts))
        } else {
            None
        };

        Ok(Stmt::If(IfStmt {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn while_statement(&mut self) -> Result<Stmt, VeltranoError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after while condition")?;

        let body_stmts = self.statement()?;
        let body = Parser::nonempty_to_stmt(body_stmts);

        Ok(Stmt::While(WhileStmt { condition, body }))
    }

    fn return_statement(&mut self) -> Result<Stmt, VeltranoError> {
        let value = if self.check(&TokenType::Newline) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_newline()?;
        Ok(Stmt::Return(value))
    }

    fn block_statement(&mut self) -> Result<Stmt, VeltranoError> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            
            // First, always check for comments to prevent them from being consumed by primary()
            if matches!(
                self.peek().token_type,
                TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _)
            ) {
                // Handle all consecutive comments
                while let Some(comment_stmt) = self.try_parse_comment() {
                    statements.push(comment_stmt);
                }
                continue;
            }
            
            // Skip any newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            statements.extend(self.declaration()?.into_iter());
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> Result<NonEmpty<Stmt>, VeltranoError> {
        let expr = self.expression()?;
        let inline_comment = self.consume_newline()?;

        let expr_stmt = Stmt::Expression(expr);

        // Add inline comment as a separate statement if present
        if let Some((content, whitespace)) = inline_comment {
            let comment = Stmt::Comment(CommentStmt {
                content,
                is_block_comment: false,
                preceding_whitespace: whitespace,
                context: CommentContext::EndOfLine,
            });
            Ok(NonEmpty::from((expr_stmt, vec![comment])))
        } else {
            Ok(NonEmpty::singleton(expr_stmt))
        }
    }

    fn parse_type(&mut self) -> Result<Located<VeltranoType>, VeltranoError> {
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
