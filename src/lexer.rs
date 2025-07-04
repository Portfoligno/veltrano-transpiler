//! Lexical analysis for the Veltrano language
//!
//! This module implements the lexer (tokenizer) that converts raw Veltrano source
//! code into a stream of tokens for the parser. It handles all token types including
//! keywords, identifiers, literals, operators, and comments.

use crate::ast::CommentContext;
use crate::config::Config;

/// Number of spaces per indentation level
const SPACES_PER_INDENT: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Fun,
    Val,
    If,
    Else,
    While,
    For,
    Return,
    True,
    False,
    Null,
    Class,
    Import,
    As,
    Data,

    // Identifiers and literals
    Identifier(String),
    IntLiteral(i64),
    StringLiteral(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And, // && operator
    Or,  // || operator

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Dot,
    Arrow,

    // Comments (with content, preceding whitespace, and context)
    LineComment(String, String, CommentContext), // (content, preceding_whitespace, context)
    BlockComment(String, String, CommentContext), // (content, preceding_whitespace, context)

    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    brace_depth: usize,     // Track nesting level of braces
    paren_depth: usize,     // Track nesting level of parentheses
    at_line_start: bool,    // Whether we're at the start of a line (after newline)
    last_token_line: usize, // Track the line of the last non-whitespace token
    config: Config,
}

impl Lexer {
    pub fn with_config(input: String, config: Config) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            brace_depth: 0,
            paren_depth: 0,
            at_line_start: true, // Start at beginning of first line
            last_token_line: 0,
            config,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let whitespace = self.collect_whitespace();
            if self.is_at_end() {
                break;
            }

            if let Some(mut token) = self.next_token() {
                // Clear at_line_start flag when we encounter any token
                self.at_line_start = false;

                // Determine comment context based on line position
                let comment_context =
                    if token.line == self.last_token_line && self.last_token_line > 0 {
                        CommentContext::EndOfLine
                    } else {
                        CommentContext::OwnLine
                    };

                // Add whitespace to comment tokens, stripping base indentation
                match &mut token.token_type {
                    TokenType::LineComment(_, ws, ctx) => {
                        // For comments, strip the expected base indentation based on brace depth
                        *ws = self.strip_base_indentation(&whitespace);
                        *ctx = comment_context;
                        if self.config.preserve_comments {
                            tokens.push(token);
                        }
                    }
                    TokenType::BlockComment(_, ws, ctx) => {
                        // For comments, strip the expected base indentation based on brace depth
                        *ws = self.strip_base_indentation(&whitespace);
                        *ctx = comment_context;
                        if self.config.preserve_comments {
                            tokens.push(token);
                        }
                    }
                    TokenType::Newline => {
                        // Don't update last_token_line for newlines
                        tokens.push(token);
                    }
                    _ => {
                        // Update last_token_line for all other non-whitespace tokens
                        self.last_token_line = token.line;
                        tokens.push(token);
                    }
                }
            }
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            line: self.line,
            column: self.column,
        });

        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column;

        let ch = match self.advance() {
            Some(ch) => ch,
            None => return None, // End of input
        };

        let token_type = match ch {
            '(' => {
                self.paren_depth += 1;
                TokenType::LeftParen
            }
            ')' => {
                if self.paren_depth > 0 {
                    self.paren_depth -= 1;
                }
                TokenType::RightParen
            }
            '{' => {
                self.brace_depth += 1;
                TokenType::LeftBrace
            }
            '}' => {
                if self.brace_depth > 0 {
                    self.brace_depth -= 1;
                }
                TokenType::RightBrace
            }
            ':' => TokenType::Colon,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '+' => TokenType::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => TokenType::Star,
            '/' => {
                if self.peek() == Some('/') {
                    // Line comment
                    self.advance(); // consume second '/'
                    let comment = self.read_line_comment();
                    TokenType::LineComment(comment, String::new(), CommentContext::OwnLine)
                } else if self.peek() == Some('*') {
                    // Block comment
                    self.advance(); // consume '*'
                    let comment = self.read_block_comment();
                    TokenType::BlockComment(comment, String::new(), CommentContext::OwnLine)
                } else {
                    TokenType::Slash
                }
            }
            '%' => TokenType::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    return None; // Invalid token
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenType::And
                } else {
                    return None; // Single & is not valid
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenType::Or
                } else {
                    return None; // Single | is not valid
                }
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true; // Mark that we're now at the start of a new line
                TokenType::Newline
            }
            '"' => {
                let string_value = self.read_string();
                TokenType::StringLiteral(string_value)
            }
            _ if ch.is_ascii_digit() => {
                let number = self.read_number(ch);
                TokenType::IntLiteral(number)
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                let identifier = self.read_identifier(ch);
                self.keyword_or_identifier(identifier)
            }
            _ => return None, // Invalid character
        };

        Some(Token {
            token_type,
            line: start_line,
            column: start_column,
        })
    }

    fn keyword_or_identifier(&self, text: String) -> TokenType {
        match text.as_str() {
            "fun" => TokenType::Fun,
            "val" => TokenType::Val,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "class" => TokenType::Class,
            "import" => TokenType::Import,
            "as" => TokenType::As,
            "data" => TokenType::Data,
            _ => TokenType::Identifier(text),
        }
    }

    fn read_string(&mut self) -> String {
        let value = self.read_while(|ch| ch != '"');

        if !self.is_at_end() {
            self.advance(); // Consume closing quote
        }

        value
    }

    fn read_number(&mut self, first_digit: char) -> i64 {
        let mut value = String::from(first_digit);
        value.push_str(&self.read_while(|c| c.is_ascii_digit()));
        value.parse().unwrap_or(0)
    }

    fn read_identifier(&mut self, first_char: char) -> String {
        let mut value = String::from(first_char);
        value.push_str(&self.read_while(|c| c.is_ascii_alphanumeric() || c == '_'));
        value
    }

    fn read_line_comment(&mut self) -> String {
        self.read_while(|ch| ch != '\n')
    }

    fn read_block_comment(&mut self) -> String {
        let mut comment = String::new();

        while !self.is_at_end() {
            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                break;
            }

            if let Some(ch) = self.advance() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                comment.push(ch);
            }
        }

        comment
    }

    fn collect_whitespace(&mut self) -> String {
        let whitespace = self.read_while(|ch| matches!(ch, ' ' | '\r' | '\t'));

        // If we just consumed whitespace at the start of a line, clear the flag
        if self.at_line_start && !whitespace.is_empty() {
            self.at_line_start = false;
        }

        whitespace
    }

    /// Calculate expected indentation based on current context
    fn expected_indentation(&self) -> usize {
        // Base indentation from brace depth
        let base = self.brace_depth * SPACES_PER_INDENT;
        // Add extra indentation for each level of parentheses (function calls, etc.)
        base + (self.paren_depth * SPACES_PER_INDENT)
    }

    /// Strip expected base indentation from whitespace for comments
    fn strip_base_indentation(&self, whitespace: &str) -> String {
        let expected = self.expected_indentation();
        if whitespace.len() >= expected {
            // Keep only the extra whitespace beyond expected indentation
            whitespace[expected..].to_string()
        } else {
            // If less than expected, keep all (might be mid-line or special formatting)
            whitespace.to_string()
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            let ch = self.input[self.position];
            self.position += 1;
            self.column += 1;
            Some(ch)
        }
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.input[self.position])
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    fn read_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut value = String::new();

        while !self.is_at_end() {
            if let Some(ch) = self.peek() {
                if !predicate(ch) {
                    break;
                }
                if let Some(ch) = self.advance() {
                    if ch == '\n' {
                        self.line += 1;
                        self.column = 1;
                    }
                    value.push(ch);
                }
            }
        }

        value
    }
}
