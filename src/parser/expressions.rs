//! Expression parsing for the Veltrano language
//!
//! This module contains all expression parsing logic including:
//! - Primary expressions (literals, identifiers)
//! - Unary and binary operations  
//! - Function and method calls
//! - Field access

use super::Parser;
use crate::ast::{
    Argument, ArgumentComment, BinaryExpr, BinaryOp, CallExpr, Expr, FieldAccessExpr, LiteralExpr,
    MethodCallExpr, ParenthesizedExpr, UnaryExpr, UnaryOp,
};
use crate::ast_types::{Located, LocatedExpr};
use crate::error::{SourceLocation, Span, VeltranoError};
use crate::lexer::TokenType;

impl Parser {
    /// Parse an expression with operator precedence
    pub(super) fn expression(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.parse_binary_expression(Self::logical_and, &[TokenType::Or], |token_type| {
            match token_type {
                TokenType::Or => BinaryOp::Or,
                _ => unreachable!(),
            }
        })
    }

    fn logical_and(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.parse_binary_expression(Self::equality, &[TokenType::And], |token_type| {
            match token_type {
                TokenType::And => BinaryOp::And,
                _ => unreachable!(),
            }
        })
    }

    fn equality(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.parse_binary_expression(
            Self::comparison,
            &[TokenType::NotEqual, TokenType::EqualEqual],
            |token_type| match token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            },
        )
    }

    fn comparison(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.parse_binary_expression(
            Self::term,
            &[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            |token_type| match token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            },
        )
    }

    fn term(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.parse_binary_expression(
            Self::factor,
            &[TokenType::Minus, TokenType::Plus],
            |token_type| match token_type {
                TokenType::Minus => BinaryOp::Subtract,
                TokenType::Plus => BinaryOp::Add,
                _ => unreachable!(),
            },
        )
    }

    fn factor(&mut self) -> Result<LocatedExpr, VeltranoError> {
        self.parse_binary_expression(
            Self::unary,
            &[TokenType::Slash, TokenType::Star, TokenType::Percent],
            |token_type| match token_type {
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            },
        )
    }

    fn unary(&mut self) -> Result<LocatedExpr, VeltranoError> {
        if self.match_token(&TokenType::Minus) {
            let start_line = self.previous().line;
            let start_column = self.previous().column;
            // Check for double minus without separation
            if self.peek().token_type == TokenType::Minus {
                return Err(self.syntax_error(
                    "Double minus (--) is not allowed. Use -(-x) instead.".to_string(),
                ));
            }

            let operator = UnaryOp::Minus;
            let operand = Box::new(self.unary()?); // Right associative
            let end_span = operand.span.end.clone();
            return Ok(Located::new(
                Expr::Unary(UnaryExpr { operator, operand }),
                Span::new(SourceLocation::new(start_line, start_column), end_span),
            ));
        }

        self.call()
    }

    fn call(&mut self) -> Result<LocatedExpr, VeltranoError> {
        let mut expr = self.primary()?;

        loop {
            // Handle method chaining across newlines
            if !self.handle_method_chain_newlines() {
                break;
            }

            if self.match_token(&TokenType::LeftParen) {
                expr = self.parse_function_call(expr)?;
            } else if self.match_token(&TokenType::Dot) {
                expr = self.parse_member_access(expr)?;
            } else if let TokenType::LineComment(_, _, _) = &self.peek().token_type {
                if !self.handle_method_chain_comment(&mut expr) {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Handles newlines in method chains, returns false if we should stop parsing
    fn handle_method_chain_newlines(&mut self) -> bool {
        let mut newline_count = 0;
        let start_pos = self.current;

        while self.check(&TokenType::Newline) {
            newline_count += 1;
            self.advance();

            // Look ahead for a dot without consuming comments
            let mut lookahead_pos = self.current;
            while lookahead_pos < self.tokens.len() {
                match &self.tokens[lookahead_pos].token_type {
                    TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                        lookahead_pos += 1;
                    }
                    TokenType::Dot => {
                        // Found a dot, so this is a method chain. Now consume the comments.
                        while self.current < lookahead_pos {
                            self.advance();
                        }
                        break;
                    }
                    _ => {
                        // Not a comment or dot, stop looking
                        break;
                    }
                }
            }

            // If we find a dot after newline(s) and comments, continue the chain
            if self.check(&TokenType::Dot) {
                break;
            }
        }

        // If we consumed newlines but didn't find a dot, we need to backtrack
        if newline_count > 0 && !self.check(&TokenType::Dot) && !self.check(&TokenType::LeftParen) {
            // Backtrack to the position after the last consumed token before newlines
            self.current = start_pos;
            return false;
        }

        true
    }

    /// Parses a function call expression
    fn parse_function_call(&mut self, callee: LocatedExpr) -> Result<LocatedExpr, VeltranoError> {
        let mut args = Vec::new();
        let mut is_multiline = false;

        // Check if there's a newline immediately after the opening parenthesis
        if self.check(&TokenType::Newline) {
            is_multiline = true;
        }

        if !self.check(&TokenType::RightParen) {
            args = self.parse_function_arguments(&mut is_multiline)?;
        }

        // Skip any newlines and comments before the closing parenthesis
        self.skip_newlines_and_comments();

        self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;

        let id = self.next_call_id;
        self.next_call_id += 1;

        let start_span = callee.span.start.clone();
        let end_span = Span::single(SourceLocation::new(
            self.previous().line,
            self.previous().column,
        ))
        .end;
        Ok(Located::new(
            Expr::Call(CallExpr {
                callee: Box::new(callee),
                args,
                is_multiline,
                id,
            }),
            Span::new(start_span, end_span),
        ))
    }

    /// Parses function arguments including named, shorthand, and bare arguments
    fn parse_function_arguments(
        &mut self,
        is_multiline: &mut bool,
    ) -> Result<Vec<Argument>, VeltranoError> {
        let mut args = Vec::new();
        let mut comment_for_next_arg: Option<(String, String)> = None;

        loop {
            // Skip newlines and track multiline
            let had_newlines = self.skip_newlines_only();
            if had_newlines {
                *is_multiline = true;
            }

            // Check if we're at the end of arguments
            if self.check(&TokenType::RightParen) || self.is_at_end() {
                break;
            }

            // Check if we have a comment at the start of a line
            if had_newlines
                && matches!(
                    self.peek().token_type,
                    TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _)
                )
            {
                // We have a comment after newlines. Check if there's an expression after it on the same line
                let saved_pos = self.current;
                let comment = self.parse_inline_comment();

                // Check if there's a non-newline token after the comment
                let has_content_after = !self.check(&TokenType::Newline)
                    && !self.check(&TokenType::RightParen)
                    && !self.is_at_end();

                if has_content_after {
                    // There's content after the comment on the same line, so it's an inline comment
                    // Restore position and let the normal inline comment handling take over
                    self.current = saved_pos;
                } else {
                    // It's a standalone comment
                    if let Some(comment) = comment {
                        args.push(Argument::StandaloneComment(comment.0, comment.1));
                        *is_multiline = true;

                        // After a standalone comment, optionally consume a comma
                        self.match_token(&TokenType::Comma);
                        continue;
                    }
                }
            }

            // Check for inline comment before the argument (or use saved comment from after previous comma)
            let comment_before = comment_for_next_arg.take().or_else(|| {
                if matches!(
                    self.peek().token_type,
                    TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _)
                ) {
                    self.parse_inline_comment()
                } else {
                    None
                }
            });

            // Parse the argument
            let mut arg = self.parse_single_argument(comment_before)?;

            // Check for comma
            if !self.match_token(&TokenType::Comma) {
                args.push(arg);
                break;
            }

            // After a comma, capture any trailing comment before the newline
            if let Some(comment_after_comma) = self.capture_comment_preserve_newlines() {
                let is_line_comment = !comment_after_comma.0.starts_with("/*");

                // If we captured a line comment after comma, consume the newline
                if is_line_comment {
                    self.match_token(&TokenType::Newline);
                }

                // Decide where to attach the comment based on its type
                if is_line_comment {
                    // Line comments after commas belong to the current argument
                    match &mut arg {
                        Argument::Bare(_, ref mut comment)
                        | Argument::Named(_, _, ref mut comment)
                        | Argument::Shorthand(_, ref mut comment) => {
                            // If there's already an after comment, keep it
                            // Otherwise, use the comment after the comma
                            if comment.after.is_none() {
                                comment.after = Some(comment_after_comma);
                            }
                        }
                        Argument::StandaloneComment(_, _) => {
                            // Standalone comments don't have an after comment field
                        }
                    }
                } else {
                    // Block comments after commas belong to the next argument
                    comment_for_next_arg = Some(comment_after_comma);
                }
            }

            args.push(arg);
        }

        Ok(args)
    }

    /// Parses a single argument (named, shorthand, or bare)
    fn parse_single_argument(
        &mut self,
        comment_before: Option<(String, String)>,
    ) -> Result<Argument, VeltranoError> {
        // Try to parse regular argument (named, shorthand, or bare)
        if self.check(&TokenType::Dot) {
            // This is shorthand syntax (.field)
            self.advance(); // consume dot
            if let TokenType::Identifier(field_name) = &self.peek().token_type {
                let field_name = field_name.clone();
                self.advance(); // consume identifier

                // Capture comment immediately after the field name
                let comment_after = self.capture_comment_preserve_newlines();

                // If we captured a line comment, consume the newline after it
                if let Some((content, _)) = &comment_after {
                    if !content.starts_with("/*") {
                        // It's a line comment, consume the newline
                        self.match_token(&TokenType::Newline);
                    }
                }

                let comment = ArgumentComment {
                    before: comment_before,
                    after: comment_after,
                };
                Ok(Argument::Shorthand(field_name, comment))
            } else {
                Err(self
                    .syntax_error("Expected field name after '.' in shorthand syntax".to_string()))
            }
        } else if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            let next_pos = self.current + 1;
            if next_pos < self.tokens.len() && self.tokens[next_pos].token_type == TokenType::Equal
            {
                // This is a named argument
                self.advance(); // consume identifier
                self.advance(); // consume =
                let value = self.expression()?;

                // Capture comment immediately after the expression
                let comment_after = self.capture_comment_preserve_newlines();

                // If we captured a line comment, consume the newline after it
                if let Some((content, _)) = &comment_after {
                    if !content.starts_with("/*") {
                        // It's a line comment, consume the newline
                        self.match_token(&TokenType::Newline);
                    }
                }

                let comment = ArgumentComment {
                    before: comment_before,
                    after: comment_after,
                };
                Ok(Argument::Named(name, value, comment))
            } else {
                // This is a bare argument starting with an identifier
                let expr = self.expression()?;

                // Capture comment immediately after the expression
                let comment_after = self.capture_comment_preserve_newlines();

                // If we captured a line comment, consume the newline after it
                if let Some((content, _)) = &comment_after {
                    if !content.starts_with("/*") {
                        // It's a line comment, consume the newline
                        self.match_token(&TokenType::Newline);
                    }
                }

                let comment = ArgumentComment {
                    before: comment_before,
                    after: comment_after,
                };
                Ok(Argument::Bare(expr, comment))
            }
        } else {
            // This is a bare argument
            let expr = self.expression()?;

            // Capture comment immediately after the expression
            let comment_after = self.capture_comment_preserve_newlines();

            // If we captured a line comment, consume the newline after it
            if let Some((content, _)) = &comment_after {
                if !content.starts_with("/*") {
                    // It's a line comment, consume the newline
                    self.match_token(&TokenType::Newline);
                }
            }

            let comment = ArgumentComment {
                before: comment_before,
                after: comment_after,
            };
            Ok(Argument::Bare(expr, comment))
        }
    }

    fn primary(&mut self) -> Result<LocatedExpr, VeltranoError> {
        // Skip any comment tokens that appear before primary expressions
        // When preserve_comments is enabled, comments become tokens in the stream
        // NOTE: We only skip comments, not newlines, to preserve error recovery
        while matches!(
            self.peek().token_type,
            TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _)
        ) {
            self.advance();
        }

        if self.match_token(&TokenType::True) {
            let token = self.previous();
            return Ok(self.located_expr(Expr::Literal(LiteralExpr::Bool(true)), token));
        }

        if self.match_token(&TokenType::False) {
            let token = self.previous();
            return Ok(self.located_expr(Expr::Literal(LiteralExpr::Bool(false)), token));
        }

        if self.match_token(&TokenType::Null) {
            let token = self.previous();
            return Ok(self.located_expr(Expr::Literal(LiteralExpr::Null), token));
        }

        if let TokenType::IntLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            let token = self.previous();
            return Ok(self.located_expr(Expr::Literal(LiteralExpr::Int(value)), token));
        }

        if let TokenType::StringLiteral(value) = &self.peek().token_type {
            let value = value.clone();
            self.advance();
            let token = self.previous();
            return Ok(self.located_expr(Expr::Literal(LiteralExpr::String(value)), token));
        }

        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            let token = self.previous();
            // Check if this is the Unit literal
            if name == "Unit" {
                return Ok(self.located_expr(Expr::Literal(LiteralExpr::Unit), token));
            }
            return Ok(self.located_expr(Expr::Identifier(name), token));
        }

        if self.match_token(&TokenType::LeftParen) {
            let start_token = self.previous();
            let start_loc = SourceLocation::new(start_token.line, start_token.column);

            // Capture comments after opening paren
            let open_paren_comment = self.capture_comment_sequence();

            let expr = self.expression()?;

            // Capture comments before closing paren
            let close_paren_comment = self.capture_comment_sequence();

            let end_token =
                self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            let end_loc = SourceLocation::new(end_token.line, end_token.column);

            let paren_expr = Expr::Parenthesized(ParenthesizedExpr {
                expr: Box::new(expr),
                open_paren_comment,
                close_paren_comment,
            });

            return Ok(self.located_expr_with_span(paren_expr, start_loc, end_loc));
        }

        Err(self.unexpected_token("expression"))
    }

    fn parse_binary_expression<F, M>(
        &mut self,
        next: F,
        operators: &[TokenType],
        map_operator: M,
    ) -> Result<LocatedExpr, VeltranoError>
    where
        F: Fn(&mut Self) -> Result<LocatedExpr, VeltranoError>,
        M: Fn(&TokenType) -> BinaryOp,
    {
        let mut expr = next(self)?;

        loop {
            // Look ahead to see if we have an operator (possibly after comments)
            let mut lookahead_pos = self.current;
            let mut comment_count = 0;

            // Count comments while looking for operator
            while lookahead_pos < self.tokens.len() {
                match &self.tokens[lookahead_pos].token_type {
                    TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                        comment_count += 1;
                        lookahead_pos += 1;
                    }
                    _ => break,
                }
            }

            // Check if we have one of the expected operators at the lookahead position
            let mut found_operator = false;
            if lookahead_pos < self.tokens.len() {
                for token_type in operators {
                    if std::mem::discriminant(&self.tokens[lookahead_pos].token_type)
                        == std::mem::discriminant(token_type)
                    {
                        found_operator = true;
                        break;
                    }
                }
            }

            if !found_operator {
                // No operator found, we're done with this precedence level
                // Don't consume any comments - they belong to a higher level
                break;
            }

            // Now we know this operator belongs to us, so consume any comments before it
            let comment_after_left = if comment_count > 0 {
                self.capture_comment_preserve_newlines()
            } else {
                None
            };

            // Skip any remaining comments before the operator
            while matches!(
                self.peek().token_type,
                TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _)
            ) {
                self.advance();
            }

            // Now consume the operator
            self.advance();
            let operator = map_operator(&self.previous().token_type);

            // Skip newlines and capture any comments after the operator
            let comment_after_operator = self.skip_newlines_and_capture_comment();

            let right = next(self)?;
            let start_span = expr.span.start.clone();
            let end_span = right.span.end.clone();
            expr = Located::new(
                Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    comment_after_left,
                    operator,
                    comment_after_operator,
                    right: Box::new(right),
                }),
                Span::new(start_span, end_span),
            );
        }

        Ok(expr)
    }

    /// Parses member access (method call or field access)
    fn parse_member_access(&mut self, object: LocatedExpr) -> Result<LocatedExpr, VeltranoError> {
        let field_or_method = self.consume_identifier("Expected field or method name after '.'")?;

        // Check if this is a method call (has parentheses) or field access
        if self.check(&TokenType::LeftParen) {
            self.parse_method_call(object, field_or_method)
        } else {
            // Field access
            let start_span = object.span.start.clone();
            let end_span = Span::single(SourceLocation::new(
                self.previous().line,
                self.previous().column,
            ))
            .end;
            Ok(Located::new(
                Expr::FieldAccess(FieldAccessExpr {
                    object: Box::new(object),
                    field: field_or_method,
                }),
                Span::new(start_span, end_span),
            ))
        }
    }

    /// Parses a method call
    fn parse_method_call(
        &mut self,
        object: LocatedExpr,
        method: String,
    ) -> Result<LocatedExpr, VeltranoError> {
        self.advance(); // consume '('

        let mut args = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                // Skip any newlines and comments before parsing the argument
                self.skip_newlines_and_comments();

                args.push(self.expression()?);

                // Skip any newlines and comments after the argument
                self.skip_newlines_and_comments();

                if !self.match_token(&TokenType::Comma) {
                    break;
                }

                // Skip any newlines and comments after the comma
                self.skip_newlines_and_comments();
            }
        }

        // Skip any newlines and comments before the closing parenthesis
        self.skip_newlines_and_comments();

        self.consume(
            &TokenType::RightParen,
            "Expected ')' after method arguments",
        )?;

        // Capture comment after method call without consuming statement-terminating newlines
        let comment = self.capture_comment_preserve_newlines();

        let id = self.next_call_id;
        self.next_call_id += 1;

        let start_span = object.span.start.clone();
        let end_span = Span::single(SourceLocation::new(
            self.previous().line,
            self.previous().column,
        ))
        .end;
        Ok(Located::new(
            Expr::MethodCall(MethodCallExpr {
                object: Box::new(object),
                method,
                args,
                inline_comment: comment,
                id,
            }),
            Span::new(start_span, end_span),
        ))
    }

    /// Handles inline comments in method chains
    fn handle_method_chain_comment(&mut self, expr: &mut LocatedExpr) -> bool {
        // Check if this inline comment is followed by newline + dot (method chain continuation)
        let next_pos = self.current + 1;
        let nextnext_pos = self.current + 2;
        if next_pos < self.tokens.len()
            && nextnext_pos < self.tokens.len()
            && matches!(self.tokens[next_pos].token_type, TokenType::Newline)
            && matches!(self.tokens[nextnext_pos].token_type, TokenType::Dot)
        {
            // This is a method chain comment - capture it and attach to the current expression
            if let Expr::MethodCall(ref mut method_call) = expr.node {
                // Capture the comment and attach it to the last method call
                let comment = self.parse_inline_comment();
                if method_call.inline_comment.is_none() {
                    method_call.inline_comment = comment;
                }
            } else {
                // Skip comment if it's not attached to a method call
                self.advance();
            }
            return true; // continue the loop
        }
        false // break from the loop
    }
}
