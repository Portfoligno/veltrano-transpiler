#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Fun,
    Var,
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

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Arrow,

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
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            if let Some(token) = self.next_token() {
                tokens.push(token);
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
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ';' => TokenType::Semicolon,
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
            '/' => TokenType::Slash,
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
            '\n' => {
                self.line += 1;
                self.column = 1;
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
            "var" => TokenType::Var,
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
            _ => TokenType::Identifier(text),
        }
    }

    fn read_string(&mut self) -> String {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != Some('"') {
            if let Some(ch) = self.advance() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                value.push(ch);
            }
        }

        if !self.is_at_end() {
            self.advance(); // Consume closing quote
        }

        value
    }

    fn read_number(&mut self, first_digit: char) -> i64 {
        let mut value = String::new();
        value.push(first_digit);

        while !self.is_at_end() && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            if let Some(ch) = self.advance() {
                value.push(ch);
            }
        }

        value.parse().unwrap_or(0)
    }

    fn read_identifier(&mut self, first_char: char) -> String {
        let mut value = String::new();
        value.push(first_char);

        while !self.is_at_end()
            && self
                .peek()
                .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
        {
            if let Some(ch) = self.advance() {
                value.push(ch);
            }
        }

        value
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                Some(' ') | Some('\r') | Some('\t') => {
                    self.advance();
                }
                _ => break,
            }
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
}
