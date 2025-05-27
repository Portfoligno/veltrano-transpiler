use crate::ast::*;
use crate::config::Config;

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    config: Config,
}

impl CodeGenerator {
    pub fn with_config(config: Config) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            config,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        for stmt in &program.statements {
            self.generate_statement(stmt);
        }
        self.output.clone()
    }

    fn generate_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr, inline_comment) => {
                self.indent();
                self.generate_expression(expr);
                self.output.push(';');
                self.generate_inline_comment(inline_comment);
                self.output.push('\n');
            }
            Stmt::VarDecl(var_decl, inline_comment) => {
                self.generate_var_declaration(var_decl, inline_comment);
            }
            Stmt::FunDecl(fun_decl) => {
                self.generate_function_declaration(fun_decl);
            }
            Stmt::If(if_stmt) => {
                self.generate_if_statement(if_stmt);
            }
            Stmt::While(while_stmt) => {
                self.generate_while_statement(while_stmt);
            }
            Stmt::Return(expr, inline_comment) => {
                self.indent();
                self.output.push_str("return");
                if let Some(expr) = expr {
                    self.output.push(' ');
                    self.generate_expression(expr);
                }
                self.output.push(';');
                self.generate_inline_comment(inline_comment);
                self.output.push('\n');
            }
            Stmt::Block(statements) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.generate_statement(stmt);
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Stmt::Comment(comment) => {
                if self.config.preserve_comments {
                    self.generate_comment(comment);
                }
            }
        }
    }

    fn generate_var_declaration(
        &mut self,
        var_decl: &VarDeclStmt,
        inline_comment: &Option<(String, String)>,
    ) {
        self.indent();

        self.output.push_str("let ");

        let snake_name = self.camel_to_snake_case(&var_decl.name);
        self.output.push_str(&snake_name);

        if let Some(type_annotation) = &var_decl.type_annotation {
            self.output.push_str(": ");
            self.generate_type(type_annotation);
        }

        if let Some(initializer) = &var_decl.initializer {
            self.output.push_str(" = ");
            self.generate_expression(initializer);
        }

        self.output.push(';');
        self.generate_inline_comment(inline_comment);
        self.output.push('\n');
    }

    fn generate_function_declaration(&mut self, fun_decl: &FunDeclStmt) {
        self.indent();
        self.output.push_str("fn ");
        let snake_name = self.camel_to_snake_case(&fun_decl.name);
        self.output.push_str(&snake_name);
        self.output.push('(');

        for (i, param) in fun_decl.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let snake_name = self.camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type);
        }

        self.output.push(')');

        if let Some(return_type) = &fun_decl.return_type {
            self.output.push_str(" -> ");
            self.generate_type(return_type);
        }

        self.output.push(' ');
        self.generate_statement(&fun_decl.body);
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStmt) {
        self.indent();
        self.output.push_str("if ");
        self.generate_expression(&if_stmt.condition);
        self.output.push(' ');

        self.generate_statement(&if_stmt.then_branch);

        if let Some(else_branch) = &if_stmt.else_branch {
            self.indent();
            self.output.push_str("else ");
            self.generate_statement(else_branch);
        }
    }

    fn generate_while_statement(&mut self, while_stmt: &WhileStmt) {
        self.indent();

        // Check if this is an infinite loop (while true)
        if let Expr::Literal(LiteralExpr::Bool(true)) = &while_stmt.condition {
            self.output.push_str("loop ");
        } else {
            self.output.push_str("while ");
            self.generate_expression(&while_stmt.condition);
            self.output.push(' ');
        }

        self.generate_statement(&while_stmt.body);
    }

    fn generate_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(literal) => {
                self.generate_literal(literal);
            }
            Expr::Identifier(name) => {
                let snake_name = self.camel_to_snake_case(name);
                self.output.push_str(&snake_name);
            }
            Expr::Binary(binary) => {
                self.generate_expression(&binary.left);
                self.output.push(' ');
                self.generate_binary_operator(&binary.operator);
                self.output.push(' ');
                self.generate_expression(&binary.right);
            }
            Expr::Call(call) => {
                self.generate_call_expression(call);
            }
            Expr::MethodCall(method_call) => {
                self.generate_method_call_expression(method_call);
            }
        }
    }

    fn generate_literal(&mut self, literal: &LiteralExpr) {
        match literal {
            LiteralExpr::Int(value) => {
                self.output.push_str(&value.to_string());
            }
            LiteralExpr::String(value) => {
                self.output.push('"');
                self.output.push_str(value);
                self.output.push('"');
            }
            LiteralExpr::Bool(value) => {
                self.output.push_str(&value.to_string());
            }
            LiteralExpr::Null => {
                self.output.push_str("None");
            }
        }
    }

    fn generate_binary_operator(&mut self, op: &BinaryOp) {
        let op_str = match op {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
        };
        self.output.push_str(op_str);
    }

    fn generate_type(&mut self, type_annotation: &Type) {
        // Generate reference prefix based on reference_depth
        for _ in 0..type_annotation.reference_depth {
            self.output.push('&');
        }

        // Generate the base type
        self.generate_base_type(&type_annotation.base);
    }

    fn generate_base_type(&mut self, base_type: &BaseType) {
        match base_type {
            BaseType::Int => self.output.push_str("i64"),
            BaseType::Bool => self.output.push_str("bool"),
            BaseType::Unit => self.output.push_str("()"),
            BaseType::Nothing => self.output.push_str("!"),
            BaseType::Str => self.output.push_str("str"),
            BaseType::String => self.output.push_str("String"),
            BaseType::MutRef(inner) => {
                self.output.push_str("&mut ");
                self.generate_type(inner);
            }
            BaseType::Box(inner) => {
                self.output.push_str("Box<");
                self.generate_type(inner);
                self.output.push('>');
            }
            BaseType::Custom(name) => self.output.push_str(name),
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn is_rust_macro(&self, name: &str) -> bool {
        matches!(
            name,
            "println" | "print" | "panic" | "assert" | "debug_assert"
        )
    }

    pub fn camel_to_snake_case(&self, name: &str) -> String {
        let mut result = String::new();
        let mut chars = name.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }

        result
    }

    fn generate_call_expression(&mut self, call: &CallExpr) {
        if let Expr::Identifier(name) = call.callee.as_ref() {
            if name == "MutRef" {
                // Special case: MutRef(value) becomes &mut (value.clone())
                if call.args.len() != 1 {
                    panic!(
                        "MutRef() requires exactly one argument, found {}",
                        call.args.len()
                    );
                }
                self.output.push_str("&mut (");
                self.generate_expression(&call.args[0]);
                self.output.push_str(".clone())");
            } else if self.is_rust_macro(name) {
                self.output.push_str(name);
                self.output.push('!');
                self.output.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expression(arg);
                }
                self.output.push(')');
            } else {
                self.generate_expression(&call.callee);
                self.output.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expression(arg);
                }
                self.output.push(')');
            }
        } else {
            self.generate_expression(&call.callee);
            self.output.push('(');
            for (i, arg) in call.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.generate_expression(arg);
            }
            self.output.push(')');
        }
    }

    fn generate_method_call_expression(&mut self, method_call: &MethodCallExpr) {
        if method_call.method == "ref" && method_call.args.is_empty() {
            // Special case: ownedValue.ref() becomes &ownedValue
            // This converts Own<T> to T (which is &T in Rust)
            self.output.push('&');
            self.generate_expression(&method_call.object);
        } else {
            // Regular method call: obj.method(args)
            let snake_method = self.camel_to_snake_case(&method_call.method);
            self.generate_expression(&method_call.object);
            self.output.push('.');
            self.output.push_str(&snake_method);
            self.output.push('(');

            for (i, arg) in method_call.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.generate_expression(arg);
            }

            self.output.push(')');
        }
    }

    fn generate_comment(&mut self, comment: &CommentStmt) {
        if comment.is_block_comment {
            self.indent();
            self.output.push_str(&comment.preceding_whitespace);
            self.output.push_str("/*");
            self.output.push_str(&comment.content);
            self.output.push_str("*/\n");
        } else {
            self.indent();
            self.output.push_str(&comment.preceding_whitespace);
            self.output.push_str("//");
            self.output.push_str(&comment.content);
            self.output.push('\n');
        }
    }

    fn generate_inline_comment(&mut self, inline_comment: &Option<(String, String)>) {
        if let Some((content, whitespace)) = inline_comment {
            if self.config.preserve_comments {
                self.output.push_str(whitespace);
                self.output.push_str("//");
                self.output.push_str(content);
            }
        }
    }
}
