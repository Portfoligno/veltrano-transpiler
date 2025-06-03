use crate::ast::*;
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    in_function_body: bool, // Track if we're parsing inside a function body
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            in_function_body: false,
        }
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

        // Second pass: analyze bump usage and update has_hidden_bump flags
        Self::analyze_bump_usage(&mut statements);

        Ok(Program { statements })
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
                // Skip any newlines and comments before parsing the parameter
                self.skip_newlines_and_comments();

                let param_name = self.consume_identifier("Expected parameter name")?;
                self.consume(&TokenType::Colon, "Expected ':' after parameter name")?;
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

        let inline_comment = self.consume_newline()?;

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

        self.consume_newline()?;

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
                // Skip any newlines and comments before parsing the field
                self.skip_newlines_and_comments();

                // Each field starts with 'val'
                self.consume(&TokenType::Val, "Expected 'val' before field name")?;
                let field_name = self.consume_identifier("Expected field name after 'val'")?;
                self.consume(&TokenType::Colon, "Expected ':' after field name")?;
                let field_type = self.parse_type()?;

                fields.push(DataClassField {
                    name: field_name,
                    field_type,
                });

                // Skip any newlines and comments after the field
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
            "Expected ')' after data class fields",
        )?;
        self.consume_newline()?;

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
        let value = if self.check(&TokenType::Newline) {
            None
        } else {
            Some(self.expression()?)
        };

        let inline_comment = self.consume_newline()?;
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
        let inline_comment = self.consume_newline()?;

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
            // Check if there's a dot after potential newlines and comments
            let mut newline_count = 0;
            let start_pos = self.current;

            while self.check(&TokenType::Newline) {
                newline_count += 1;
                self.advance();

                // Skip any standalone comments after the newline
                while let TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) =
                    &self.peek().token_type
                {
                    self.advance();
                }

                // If we find a dot after newline(s) and comments, continue the chain
                if self.check(&TokenType::Dot) {
                    break;
                }
            }

            // If we consumed newlines but didn't find a dot, we need to backtrack
            if newline_count > 0
                && !self.check(&TokenType::Dot)
                && !self.check(&TokenType::LeftParen)
            {
                // Put back one newline for the statement terminator
                self.current = start_pos + newline_count - 1;
                break;
            }

            if self.match_token(&TokenType::LeftParen) {
                let mut args = Vec::new();
                let mut is_multiline = false;

                // Check if there's a newline immediately after the opening parenthesis
                if self.check(&TokenType::Newline) {
                    is_multiline = true;
                }

                if !self.check(&TokenType::RightParen) {
                    loop {
                        // First check if we have a standalone comment (before skipping)
                        if let Some(standalone_comment) = self.try_parse_standalone_comment() {
                            args.push(Argument::StandaloneComment(
                                standalone_comment.0,
                                standalone_comment.1,
                            ));
                            is_multiline = true; // Standalone comments force multiline

                            // Check for comma after standalone comment
                            if self.match_token(&TokenType::Comma) {
                                continue; // Continue to next argument/comment
                            } else {
                                break; // No comma, end of arguments
                            }
                        }

                        // Skip newlines and track multiline (if no comment was found)
                        let had_newlines = self.skip_newlines_and_track_multiline();
                        if had_newlines {
                            is_multiline = true;
                        }

                        // Try to parse regular argument (named or bare)
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

                                // Capture comment immediately after the expression
                                let comment = self.skip_newlines_and_capture_comment();
                                args.push(Argument::Named(name, value, comment));
                            } else {
                                // This is a bare argument starting with an identifier
                                let expr = self.expression()?;

                                // Capture comment immediately after the expression
                                let comment = self.skip_newlines_and_capture_comment();
                                args.push(Argument::Bare(expr, comment));
                            }
                        } else {
                            // This is a bare argument
                            let expr = self.expression()?;

                            // Capture comment immediately after the expression
                            let comment = self.skip_newlines_and_capture_comment();
                            args.push(Argument::Bare(expr, comment));
                        }

                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }

                        // After comma, check for either inline comment or standalone comment
                        // First check for immediate inline comment (no newlines)
                        if let Some(inline_comment) = self.capture_comment_preserve_newlines() {
                            // This is an inline comment - assign to previous argument
                            if let Some(last_arg) = args.last_mut() {
                                match last_arg {
                                    Argument::Bare(_, ref mut existing_comment) => {
                                        if existing_comment.is_none() {
                                            *existing_comment = Some(inline_comment);
                                        }
                                    }
                                    Argument::Named(_, _, ref mut existing_comment) => {
                                        if existing_comment.is_none() {
                                            *existing_comment = Some(inline_comment);
                                        }
                                    }
                                    Argument::StandaloneComment(_, _) => {
                                        // Standalone comments can't have inline comments attached
                                    }
                                }
                            }
                        } else {
                            // No inline comment found, check for standalone comment after newlines
                            // Skip newlines only (preserve comments)
                            let had_newlines = self.skip_newlines_only();
                            if had_newlines {
                                is_multiline = true;

                                // Now check if there's a standalone comment
                                if let Some(standalone_comment) =
                                    self.try_parse_standalone_comment()
                                {
                                    args.push(Argument::StandaloneComment(
                                        standalone_comment.0,
                                        standalone_comment.1,
                                    ));
                                    // Don't consume comma here - let the next iteration handle it
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Skip any newlines and comments before the closing parenthesis
                self.skip_newlines_and_comments();

                self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;

                expr = Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    args,
                    is_multiline,
                });
            } else if self.match_token(&TokenType::Dot) {
                let field_or_method =
                    self.consume_identifier("Expected field or method name after '.'")?;

                // Check if this is a method call (has parentheses) or field access
                if self.check(&TokenType::LeftParen) {
                    // Method call
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

                    expr = Expr::MethodCall(MethodCallExpr {
                        object: Box::new(expr),
                        method: field_or_method,
                        args,
                        inline_comment: comment,
                    });
                } else {
                    // Field access
                    expr = Expr::FieldAccess(FieldAccessExpr {
                        object: Box::new(expr),
                        field: field_or_method,
                    });
                }
            } else if let TokenType::LineComment(_, _) = &self.peek().token_type {
                // Check if this inline comment is followed by newline + dot (method chain continuation)
                let next_pos = self.current + 1;
                let nextnext_pos = self.current + 2;
                if next_pos < self.tokens.len()
                    && nextnext_pos < self.tokens.len()
                    && matches!(self.tokens[next_pos].token_type, TokenType::Newline)
                    && matches!(self.tokens[nextnext_pos].token_type, TokenType::Dot)
                {
                    // This is a method chain comment - capture it and attach to the current expression
                    if let Expr::MethodCall(ref mut method_call) = expr {
                        // Capture the comment and attach it to the last method call
                        let comment = self.parse_inline_comment();
                        if method_call.inline_comment.is_none() {
                            method_call.inline_comment = comment;
                        }
                    } else {
                        // Skip comment if it's not attached to a method call
                        self.advance();
                    }
                    continue;
                }
                break;
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
        match &self.peek().token_type {
            TokenType::LineComment(content, whitespace) => {
                let content = content.clone();
                let whitespace = whitespace.clone();
                self.advance();
                Some((content, whitespace))
            }
            TokenType::BlockComment(content, whitespace) => {
                // For block comments, content is just the inner text, so wrap it with /* */
                let content = format!("/*{}*/", content);
                let whitespace = whitespace.clone();
                self.advance();
                Some((content, whitespace))
            }
            _ => None,
        }
    }

    fn consume_newline(&mut self) -> Result<Option<(String, String)>, String> {
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
                TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) => {
                    // Don't advance or consume - let the higher-level parser handle this comment
                    Ok(None)
                }
                _ => {
                    let unexpected = self.peek();
                    Err(format!(
                        "Expected newline after statement at line {}, column {}, but found {:?}",
                        unexpected.line, unexpected.column, unexpected.token_type
                    ))
                }
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

    /// Skip newlines and comments in contexts where they should be ignored (e.g., inside expressions)
    fn skip_newlines_and_comments(&mut self) {
        loop {
            if self.match_token(&TokenType::Newline) {
                // Continue to check for more newlines or comments
                continue;
            }

            // Check if there's a comment token to skip
            match &self.peek().token_type {
                TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) => {
                    self.advance(); // Skip the comment token
                }
                _ => break, // No more newlines or comments to skip
            }
        }
    }

    /// Skip newlines and comments, returning true if any newlines were found
    fn skip_newlines_and_track_multiline(&mut self) -> bool {
        let mut found_newlines = false;

        loop {
            if self.match_token(&TokenType::Newline) {
                found_newlines = true;
                continue;
            }

            // Check if there's a comment token to skip
            match &self.peek().token_type {
                TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) => {
                    self.advance(); // Skip the comment token
                }
                _ => break, // No more newlines or comments to skip
            }
        }

        found_newlines
    }

    /// Skip newlines and optionally capture inline comments
    /// Returns the first inline comment found, if any
    fn skip_newlines_and_capture_comment(&mut self) -> Option<(String, String)> {
        let mut captured_comment = None;

        loop {
            if self.match_token(&TokenType::Newline) {
                // Continue to check for more newlines or comments
                continue;
            }

            // Check if there's a comment token
            match &self.peek().token_type {
                TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) => {
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
    fn capture_comment_preserve_newlines(&mut self) -> Option<(String, String)> {
        // Only capture comment if it's immediately present (no newlines before it)
        match &self.peek().token_type {
            TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) => {
                self.parse_inline_comment()
            }
            _ => None,
        }
    }

    fn try_parse_standalone_comment(&mut self) -> Option<(String, String)> {
        // Check if there's a comment token immediately at current position
        match &self.peek().token_type {
            TokenType::LineComment(_, _) | TokenType::BlockComment(_, _) => {
                self.parse_inline_comment()
            }
            _ => None,
        }
    }

    fn skip_newlines_only(&mut self) -> bool {
        let mut found_newlines = false;

        while self.match_token(&TokenType::Newline) {
            found_newlines = true;
        }

        found_newlines
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
