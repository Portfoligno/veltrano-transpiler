//! Statement parsing for the Veltrano language
//!
//! This module contains all statement parsing logic including:
//! - Declarations (function, variable, import, data class)
//! - Control flow statements (if, while, return)
//! - Block statements
//! - Expression statements

use super::Parser;
use crate::ast::{CommentContext, CommentStmt};
use crate::ast::{
    DataClassField, DataClassStmt, FunDeclStmt, IfStmt, ImportStmt, Parameter, Stmt, VarDeclStmt,
    WhileStmt,
};
use crate::error::{ErrorKind, VeltranoError};
use crate::lexer::TokenType;
use nonempty::NonEmpty;

impl Parser {
    pub(super) fn declaration(&mut self) -> Result<NonEmpty<Stmt>, VeltranoError> {
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
        // Capture the location of the import keyword
        let import_token = self.previous();
        let location = crate::error::SourceLocation::new(import_token.line, import_token.column);

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
            location,
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

    pub(super) fn statement(&mut self) -> Result<NonEmpty<Stmt>, VeltranoError> {
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

    pub(super) fn block_statement(&mut self) -> Result<Stmt, VeltranoError> {
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
}
