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
        } else if self.match_token(&TokenType::Val) {
            self.var_declaration()
        } else if self.match_token(&TokenType::Import) {
            Ok(vec![self.import_declaration()?])
        } else if self.match_token(&TokenType::Data) {
            Ok(vec![self.data_class_declaration()?])
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

    fn var_declaration(&mut self) -> Result<Vec<Stmt>, String> {
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
                type_annotation,
                initializer,
            },
            inline_comment,
        )])
    }

    fn import_declaration(&mut self) -> Result<Stmt, String> {
        // import Type.method [as alias]
        let type_name = self.consume_identifier("Expected type name after 'import'")?;
        self.consume(&TokenType::Dot, "Expected '.' after type name")?;
        let method_name = self.consume_identifier("Expected method name after '.'")?;

        let alias = if self.match_token(&TokenType::As) {
            Some(self.consume_identifier("Expected alias name after 'as'")?)
        } else {
            None
        };

        self.consume_newline_or_semicolon()?;

        Ok(Stmt::Import(ImportStmt {
            type_name,
            method_name,
            alias,
        }))
    }

    fn data_class_declaration(&mut self) -> Result<Stmt, String> {
        // data class ClassName(val field1: Type1, val field2: Type2, ...)
        self.consume(&TokenType::Class, "Expected 'class' after 'data'")?;
        let name = self.consume_identifier("Expected data class name after 'data class'")?;

        self.consume(&TokenType::LeftParen, "Expected '(' after data class name")?;

        let mut fields = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                // Each field starts with 'val'
                self.consume(&TokenType::Val, "Expected 'val' before field name")?;
                let field_name = self.consume_identifier("Expected field name after 'val'")?;
                self.consume(&TokenType::Colon, "Expected ':' after field name")?;
                let field_type = self.parse_type()?;

                fields.push(DataClassField {
                    name: field_name,
                    field_type,
                });

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(
            &TokenType::RightParen,
            "Expected ')' after data class fields",
        )?;
        self.consume_newline_or_semicolon()?;

        Ok(Stmt::DataClass(DataClassStmt { name, fields }))
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

    fn comparison(&mut self) -> Result<Expr, String> {
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

    fn term(&mut self) -> Result<Expr, String> {
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

    fn factor(&mut self) -> Result<Expr, String> {
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

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenType::Minus) {
            // Check for double minus without separation
            if self.peek().token_type == TokenType::Minus {
                return Err("Double minus (--) is not allowed. Use -(-x) instead.".to_string());
            }

            let operator = UnaryOp::Minus;
            let operand = Box::new(self.unary()?); // Right associative
            return Ok(Expr::Unary(UnaryExpr { operator, operand }));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                let mut args = Vec::new();

                if !self.check(&TokenType::RightParen) {
                    loop {
                        // Try to parse named argument (name = expr)
                        if let TokenType::Identifier(name) = &self.peek().token_type {
                            let name = name.clone();
                            let next_pos = self.current + 1;
                            if next_pos < self.tokens.len()
                                && self.tokens[next_pos].token_type == TokenType::Equal
                            {
                                // This is a named argument
                                self.advance(); // consume identifier
                                self.advance(); // consume =
                                let value = self.expression()?;
                                args.push(Argument::Named(name, value));
                            } else {
                                // This is a positional argument starting with an identifier
                                let expr = self.expression()?;
                                args.push(Argument::Positional(expr));
                            }
                        } else {
                            // This is a positional argument
                            let expr = self.expression()?;
                            args.push(Argument::Positional(expr));
                        }

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
            // Check if this is the Unit literal
            if name == "Unit" {
                return Ok(Expr::Literal(LiteralExpr::Unit));
            }
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
                "Int" => Ok(Type::owned(BaseType::Int)),
                "Str" => Ok(Type {
                    base: BaseType::Str,
                    reference_depth: 1,
                }), // reference-by-default
                "String" => Ok(Type {
                    base: BaseType::String,
                    reference_depth: 1,
                }), // reference-by-default
                "Bool" => Ok(Type::owned(BaseType::Bool)),
                "Unit" => Ok(Type::owned(BaseType::Unit)),
                "Nothing" => Ok(Type::owned(BaseType::Nothing)),
                "Ref" => self.parse_ref_type(),
                "Own" => self.parse_own_type(),
                "MutRef" => self.parse_mutref_type(),
                "Box" => self.parse_box_type(),
                _ => Ok(Type {
                    base: BaseType::Custom(type_name),
                    reference_depth: 1,
                }), // reference-by-default
            }
        } else {
            Err("Expected type name".to_string())
        }
    }

    fn parse_ref_type(&mut self) -> Result<Type, String> {
        self.consume(&TokenType::Less, "Expected '<' after Ref")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        // Ref<T> adds one more reference level to T
        Ok(Type {
            base: inner_type.base,
            reference_depth: inner_type.reference_depth + 1,
        })
    }

    fn parse_own_type(&mut self) -> Result<Type, String> {
        self.consume(&TokenType::Less, "Expected '<' after Own")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;

        // Validate that Own<T> is not used with invalid types
        match &inner_type.base {
            BaseType::Int | BaseType::Bool | BaseType::Unit => {
                return Err(format!(
                    "Cannot use Own<{:?}>. {:?} is already owned.",
                    inner_type.base, inner_type.base
                ));
            }
            BaseType::MutRef(_) => {
                return Err("Cannot use Own<MutRef<T>>. MutRef<T> is already owned.".to_string());
            }
            BaseType::Box(_) => {
                return Err("Cannot use Own<Box<T>>. Box<T> is already owned.".to_string());
            }
            _ => {}
        }

        // Own<T> subtracts 1 from reference_depth
        if inner_type.reference_depth == 0 {
            return Err("Cannot use Own<> on already owned type.".to_string());
        }

        Ok(Type {
            base: inner_type.base,
            reference_depth: inner_type.reference_depth - 1,
        })
    }

    fn parse_mutref_type(&mut self) -> Result<Type, String> {
        self.consume(&TokenType::Less, "Expected '<' after MutRef")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(Type {
            base: BaseType::MutRef(Box::new(inner_type)),
            reference_depth: 0, // MutRef<T> is always owned
        })
    }

    fn parse_box_type(&mut self) -> Result<Type, String> {
        self.consume(&TokenType::Less, "Expected '<' after Box")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(Type {
            base: BaseType::Box(Box::new(inner_type)),
            reference_depth: 0, // Box<T> is always owned
        })
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
                let unexpected = self.peek();
                Err(format!(
                    "Expected newline or semicolon after statement at line {}, column {}, but found {:?}",
                    unexpected.line, unexpected.column, unexpected.token_type
                ))
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

    fn parse_binary_expression<F, M>(
        &mut self,
        next: F,
        operators: &[TokenType],
        map_operator: M,
    ) -> Result<Expr, String>
    where
        F: Fn(&mut Self) -> Result<Expr, String>,
        M: Fn(&TokenType) -> BinaryOp,
    {
        let mut expr = next(self)?;

        while self.match_tokens(operators) {
            let operator = map_operator(&self.previous().token_type);
            let right = next(self)?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }
}
