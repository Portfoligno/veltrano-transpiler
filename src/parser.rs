use crate::ast::*;
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
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

            match self.declaration() {
                Ok(stmts) => statements.extend(stmts),
                Err(err) => return Err(err),
            }
        }

        Ok(Program { statements })
    }

    fn declaration(&mut self) -> Result<Vec<Stmt>, String> {
        if self.match_token(&TokenType::Fun) {
            Ok(vec![self.function_declaration()?])
        } else if self.match_token(&TokenType::Var) {
            self.var_declaration(true)
        } else if self.match_token(&TokenType::Val) {
            self.var_declaration(false)
        } else {
            self.statement()
        }
    }

    fn function_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Expected function name")?;

        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;
                self.consume(&TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;

                params.push(Parameter {
                    name: param_name,
                    param_type,
                });

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = Box::new(self.block_statement()?);

        Ok(Stmt::FunDecl(FunDeclStmt {
            name,
            params,
            return_type,
            body,
        }))
    }

    fn var_declaration(&mut self, is_mutable: bool) -> Result<Vec<Stmt>, String> {
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

        let inline_comment = self.consume_newline_or_semicolon()?;

        Ok(vec![Stmt::VarDecl(
            VarDeclStmt {
                name,
                is_mutable,
                type_annotation,
                initializer,
            },
            inline_comment,
        )])
    }

    fn statement(&mut self) -> Result<Vec<Stmt>, String> {
        if self.match_token(&TokenType::If) {
            Ok(vec![self.if_statement()?])
        } else if self.match_token(&TokenType::While) {
            Ok(vec![self.while_statement()?])
        } else if self.match_token(&TokenType::Return) {
            Ok(vec![self.return_statement()?])
        } else if self.match_token(&TokenType::LeftBrace) {
            Ok(vec![self.block_statement()?])
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;

        let then_stmts = self.statement()?;
        let then_branch = if then_stmts.len() == 1 {
            Box::new(then_stmts.into_iter().next().unwrap())
        } else {
            Box::new(Stmt::Block(then_stmts))
        };

        let else_branch = if self.match_token(&TokenType::Else) {
            let else_stmts = self.statement()?;
            Some(if else_stmts.len() == 1 {
                Box::new(else_stmts.into_iter().next().unwrap())
            } else {
                Box::new(Stmt::Block(else_stmts))
            })
        } else {
            None
        };

        Ok(Stmt::If(IfStmt {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after while condition")?;

        let body_stmts = self.statement()?;
        let body = if body_stmts.len() == 1 {
            Box::new(body_stmts.into_iter().next().unwrap())
        } else {
            Box::new(Stmt::Block(body_stmts))
        };

        Ok(Stmt::While(WhileStmt { condition, body }))
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = if self.check(&TokenType::Newline) || self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        let inline_comment = self.consume_newline_or_semicolon()?;
        Ok(Stmt::Return(value, inline_comment))
    }

    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            // Handle comment tokens in blocks
            if let Some(comment_stmt) = self.try_parse_comment() {
                statements.push(comment_stmt);
                continue;
            }

            statements.extend(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> Result<Vec<Stmt>, String> {
        let expr = self.expression()?;
        let inline_comment = self.consume_newline_or_semicolon()?;

        Ok(vec![Stmt::Expression(expr, inline_comment)])
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::NotEqual, TokenType::EqualEqual]) {
            let operator = match self.previous().token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = match self.previous().token_type {
                TokenType::Minus => BinaryOp::Subtract,
                TokenType::Plus => BinaryOp::Add,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.call()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let operator = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.call()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                let mut args = Vec::new();

                if !self.check(&TokenType::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;

                expr = Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    args,
                });
            } else if self.match_token(&TokenType::Dot) {
                let method_name = self.consume_identifier("Expected method name after '.'")?;

                self.consume(&TokenType::LeftParen, "Expected '(' after method name")?;

                let mut args = Vec::new();
                if !self.check(&TokenType::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(
                    &TokenType::RightParen,
                    "Expected ')' after method arguments",
                )?;

                expr = Expr::MethodCall(MethodCallExpr {
                    object: Box::new(expr),
                    method: method_name,
                    args,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenType::True) {
            return Ok(Expr::Literal(LiteralExpr::Bool(true)));
        }

        if self.match_token(&TokenType::False) {
            return Ok(Expr::Literal(LiteralExpr::Bool(false)));
        }

        if self.match_token(&TokenType::Null) {
            return Ok(Expr::Literal(LiteralExpr::Null));
        }

        if let TokenType::IntLiteral(value) = &self.peek().token_type {
            let value = *value;
            self.advance();
            return Ok(Expr::Literal(LiteralExpr::Int(value)));
        }

        if let TokenType::StringLiteral(value) = &self.peek().token_type {
            let value = value.clone();
            self.advance();
            return Ok(Expr::Literal(LiteralExpr::String(value)));
        }

        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            return Ok(Expr::Identifier(name));
        }

        if self.match_token(&TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        Err(format!("Unexpected token: {:?}", self.peek()))
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if let TokenType::Identifier(type_name) = &self.peek().token_type {
            let type_name = type_name.clone();
            self.advance();

            match type_name.as_str() {
                "Int" => Ok(Type::Int),
                "Str" => Ok(Type::Str),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "Unit" => Ok(Type::Unit),
                "Nothing" => Ok(Type::Nothing),
                "Ref" => self.parse_generic_type(Type::Ref),
                "Box" => self.parse_generic_type(Type::Box),
                _ => Ok(Type::Custom(type_name)),
            }
        } else {
            Err("Expected type name".to_string())
        }
    }

    fn parse_generic_type<F>(&mut self, constructor: F) -> Result<Type, String>
    where
        F: FnOnce(Box<Type>) -> Type,
    {
        self.consume(&TokenType::Less, "Expected '<' after generic type")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(constructor(Box::new(inner_type)))
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(message.to_string())
        }
    }

    fn parse_inline_comment(&mut self) -> Option<(String, String)> {
        if let TokenType::LineComment(content, whitespace) = &self.peek().token_type {
            let content = content.clone();
            let whitespace = whitespace.clone();
            self.advance();
            Some((content, whitespace))
        } else {
            None
        }
    }

    fn consume_newline_or_semicolon(&mut self) -> Result<Option<(String, String)>, String> {
        // First check for semicolon
        if self.check(&TokenType::Semicolon) {
            self.advance(); // consume semicolon
            // Now check for inline comment after semicolon
            Ok(self.parse_inline_comment())
        } else {
            // Check for inline comment before newline (no semicolon case)
            let inline_comment = self.parse_inline_comment();

            if self.check(&TokenType::Newline) {
                self.advance();
                Ok(inline_comment)
            } else if self.is_at_end() || self.check(&TokenType::RightBrace) {
                Ok(inline_comment)
            } else {
                Err("Expected newline or semicolon".to_string())
            }
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(message.to_string())
        }
    }

    fn try_parse_comment(&mut self) -> Option<Stmt> {
        match &self.peek().token_type {
            TokenType::LineComment(content, whitespace) => {
                let content = content.clone();
                let whitespace = whitespace.clone();
                self.advance();
                Some(Stmt::Comment(CommentStmt {
                    content,
                    is_block_comment: false,
                    preceding_whitespace: whitespace,
                }))
            }
            TokenType::BlockComment(content, whitespace) => {
                let content = content.clone();
                let whitespace = whitespace.clone();
                self.advance();
                Some(Stmt::Comment(CommentStmt {
                    content,
                    is_block_comment: true,
                    preceding_whitespace: whitespace,
                }))
            }
            _ => None,
        }
    }
}
