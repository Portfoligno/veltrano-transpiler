//! Parser utility functions for token manipulation and common operations

use crate::ast::Stmt;
use crate::ast_types::{CommentStmt, Located, LocatedExpr};
use crate::comments::{Comment, CommentStyle};
use crate::error::{SourceLocation, Span, VeltranoError};
use crate::lexer::{Token, TokenType};
use super::Parser;
use nonempty::NonEmpty;

impl Parser {
    /// Create a Located expression with span from a single token
    pub(super) fn located_expr(&self, expr: crate::ast::Expr, token: &Token) -> LocatedExpr {
        Located::new(
            expr,
            Span::single(SourceLocation::new(token.line, token.column)),
        )
    }

    /// Create a Located expression with span from start to end tokens
    pub(super) fn _located_expr_span(&self, expr: crate::ast::Expr, start: &Token, end: &Token) -> LocatedExpr {
        Located::new(
            expr,
            Span::new(
                SourceLocation::new(start.line, start.column),
                SourceLocation::new(end.line, end.column),
            ),
        )
    }

    /// Check if the current token matches a type (without consuming)
    pub(super) fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    /// Advance to the next token
    pub(super) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Check if we're at the end of tokens
    pub(super) fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    /// Peek at the current token
    pub(super) fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Get the previous token
    pub(super) fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Consume a token of the expected type or return an error
    pub(super) fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, VeltranoError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.unexpected_token(message))
        }
    }

    /// Match and consume a token type
    pub(super) fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume an identifier token
    pub(super) fn consume_identifier(&mut self, message: &str) -> Result<String, VeltranoError> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.syntax_error(message.to_string()))
        }
    }

    /// Convert a non-empty vector of statements to a single statement.
    /// If the vector contains exactly one statement, extract it.
    /// Otherwise, wrap the statements in a Block.
    pub(super) fn nonempty_to_stmt(statements: NonEmpty<Stmt>) -> Box<Stmt> {
        if statements.len() == 1 {
            Box::new(statements.head)
        } else {
            Box::new(Stmt::Block(statements.into()))
        }
    }

    /// Skip newlines only (no comments)
    pub(super) fn skip_newlines_only(&mut self) -> bool {
        let mut found_newlines = false;

        while self.match_token(&TokenType::Newline) {
            found_newlines = true;
        }

        found_newlines
    }

    /// Skip newlines and comments in contexts where they should be ignored
    pub(super) fn skip_newlines_and_comments(&mut self) {
        loop {
            if self.match_token(&TokenType::Newline) {
                // Continue to check for more newlines or comments
                continue;
            }

            // Check if there's a comment token to skip
            match &self.peek().token_type {
                TokenType::LineComment(_content, _, _) => {
                    self.advance(); // Skip the comment token
                }
                TokenType::BlockComment(_content, _, _) => {
                    self.advance(); // Skip the comment token
                }
                _ => break, // No more newlines or comments to skip
            }
        }
    }

    /// Parse an inline comment if present
    pub(super) fn parse_inline_comment(&mut self) -> Option<(String, String)> {
        match &self.peek().token_type {
            TokenType::LineComment(content, whitespace, _context) => {
                // Lexer returns line comment content without // prefix
                let comment = Comment::new(content.clone(), whitespace.clone(), CommentStyle::Line);
                self.advance();
                Some(comment.to_tuple())
            }
            TokenType::BlockComment(content, whitespace, _context) => {
                // For block comments, content is just the inner text, so wrap it with /* */
                let comment = Comment::new(
                    format!("/*{}*/", content),
                    whitespace.clone(),
                    CommentStyle::Block,
                );
                self.advance();
                Some(comment.to_tuple())
            }
            _ => None,
        }
    }

    /// Try to parse a comment as a statement
    pub(super) fn try_parse_comment(&mut self) -> Option<Stmt> {
        match &self.peek().token_type {
            TokenType::LineComment(content, whitespace, lexer_context) => {
                let content = content.clone();
                let whitespace = whitespace.clone();
                let context = lexer_context.clone();
                self.advance();
                Some(Stmt::Comment(CommentStmt {
                    content,
                    is_block_comment: false,
                    preceding_whitespace: whitespace,
                    context,
                }))
            }
            TokenType::BlockComment(content, whitespace, lexer_context) => {
                let content = content.clone();
                let whitespace = whitespace.clone();
                let context = lexer_context.clone();
                self.advance();
                Some(Stmt::Comment(CommentStmt {
                    content,
                    is_block_comment: true,
                    preceding_whitespace: whitespace,
                    context,
                }))
            }
            _ => None,
        }
    }

    /// Skip newlines and optionally capture inline comments
    /// Returns the first inline comment found, if any
    pub(super) fn skip_newlines_and_capture_comment(&mut self) -> Option<(String, String)> {
        let mut captured_comment = None;

        loop {
            if self.match_token(&TokenType::Newline) {
                // Continue to check for more newlines or comments
                continue;
            }

            // Check if there's a comment token
            match &self.peek().token_type {
                TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                    // Capture the first comment we encounter
                    if captured_comment.is_none() {
                        captured_comment = self.parse_inline_comment();
                    } else {
                        // Skip additional comments
                        self.advance();
                    }
                }
                _ => break, // No more newlines or comments
            }
        }

        captured_comment
    }

    /// Capture inline comment without consuming statement-terminating newlines
    /// This is used for method chains where we need to preserve statement boundaries
    pub(super) fn capture_comment_preserve_newlines(&mut self) -> Option<(String, String)> {
        // Only capture comment if it's immediately present (no newlines before it)
        match &self.peek().token_type {
            TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                self.parse_inline_comment()
            }
            _ => None,
        }
    }

    /// Try to parse a standalone comment
    pub(super) fn try_parse_standalone_comment(&mut self) -> Option<(String, String)> {
        // Check if there's a comment token immediately at current position
        match &self.peek().token_type {
            TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                self.parse_inline_comment()
            }
            _ => None,
        }
    }

    /// Consume a newline, optionally with a comment
    pub(super) fn consume_newline(&mut self) -> Result<Option<(String, String)>, VeltranoError> {
        if self.check(&TokenType::Newline) {
            // Check for inline comment before newline
            let inline_comment = self.parse_inline_comment();
            self.advance();
            Ok(inline_comment)
        } else if self.is_at_end() || self.check(&TokenType::RightBrace) {
            // Check for inline comment at end of input or block
            let inline_comment = self.parse_inline_comment();
            Ok(inline_comment)
        } else {
            // If we encounter a standalone comment token (when preserve_comments is enabled),
            // we should not treat it as an error since it will be handled by the higher-level parser
            match &self.peek().token_type {
                TokenType::LineComment(_, _, _) | TokenType::BlockComment(_, _, _) => {
                    // Don't advance or consume - let the higher-level parser handle this comment
                    Ok(None)
                }
                _ => {
                    let unexpected = self.peek();
                    Err(self.error_at_token(
                        crate::error::ErrorKind::SyntaxError,
                        format!(
                            "Expected newline after statement at line {}, column {}, but found {:?}",
                            unexpected.line, unexpected.column, unexpected.token_type
                        ),
                        unexpected
                    ))
                }
            }
        }
    }
}
