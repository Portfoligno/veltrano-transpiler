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
            
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => return Err(err),
            }
        }
        
        Ok(Program { statements })
    }
    
    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenType::Fun) {
            self.function_declaration()
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
    
    fn var_declaration(&mut self, is_mutable: bool) -> Result<Stmt, String> {
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
        
        self.consume_newline_or_semicolon()?;
        
        Ok(Stmt::VarDecl(VarDeclStmt {
            name,
            is_mutable,
            type_annotation,
            initializer,
        }))
    }
    
    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenType::If) {
            self.if_statement()
        } else if self.match_token(&TokenType::While) {
            self.while_statement()
        } else if self.match_token(&TokenType::Return) {
            self.return_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            Ok(self.block_statement()?)
        } else {
            self.expression_statement()
        }
    }
    
    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;
        
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&TokenType::Else) {
            Some(Box::new(self.statement()?))
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
        
        let body = Box::new(self.statement()?);
        
        Ok(Stmt::While(WhileStmt { condition, body }))
    }
    
    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = if self.check(&TokenType::Newline) || self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        
        self.consume_newline_or_semicolon()?;
        Ok(Stmt::Return(value))
    }
    
    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }
            
            statements.push(self.declaration()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Stmt::Block(statements))
    }
    
    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume_newline_or_semicolon()?;
        Ok(Stmt::Expression(expr))
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
        
        while self.match_token(&TokenType::LeftParen) {
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
            
            let parsed_type = match type_name.as_str() {
                "Int" => Type::Int,
                "String" => Type::String,
                "Bool" => Type::Bool,
                "Unit" => Type::Unit,
                _ => Type::Custom(type_name),
            };
            
            Ok(parsed_type)
        } else {
            Err("Expected type name".to_string())
        }
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
    
    fn consume_newline_or_semicolon(&mut self) -> Result<(), String> {
        if self.check(&TokenType::Newline) || self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(())
        } else if self.is_at_end() || self.check(&TokenType::RightBrace) {
            Ok(())
        } else {
            Err("Expected newline or semicolon".to_string())
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
}