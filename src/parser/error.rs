//! Parser error handling and recovery utilities

use crate::error::{ErrorKind, SourceLocation, Span, VeltranoError};
use crate::lexer::{Token, TokenType};
use super::Parser;

impl Parser {
    /// Synchronize parser after an error to a known good state
    pub(super) fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // Stop at statement boundaries
            if self.previous().token_type == TokenType::Newline {
                return;
            }

            // Stop at keywords that begin statements
            match self.peek().token_type {
                TokenType::Fun
                | TokenType::Val
                | TokenType::If
                | TokenType::For
                | TokenType::While
                | TokenType::Return
                | TokenType::Data
                | TokenType::Import => return,
                _ => {}
            }

            self.advance();
        }
    }

    /// Create an error with the given kind and message at the current token
    pub(super) fn error(&self, kind: ErrorKind, message: String) -> VeltranoError {
        let token = self.peek();
        VeltranoError::new(kind, message)
            .with_span(Span::single(SourceLocation::new(token.line, token.column)))
    }

    /// Create an error at a specific token
    pub(super) fn error_at_token(&self, kind: ErrorKind, message: String, token: &Token) -> VeltranoError {
        VeltranoError::new(kind, message)
            .with_span(Span::single(SourceLocation::new(token.line, token.column)))
    }

    /// Create a syntax error with the given message
    pub(super) fn syntax_error(&self, message: String) -> VeltranoError {
        self.error(ErrorKind::SyntaxError, message)
    }

    /// Create an unexpected token error
    pub(super) fn unexpected_token(&self, expected: &str) -> VeltranoError {
        let token = self.peek();
        if token.token_type == TokenType::Eof {
            self.error(
                ErrorKind::UnexpectedEof,
                format!("Expected {}, found EOF", expected),
            )
        } else {
            self.error(
                ErrorKind::UnexpectedToken,
                format!("Expected {}, found {:?}", expected, token.token_type),
            )
        }
    }
}