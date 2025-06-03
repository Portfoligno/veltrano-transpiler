use crate::ast::*;
use crate::config::Config;
use std::collections::{HashMap, HashSet};

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    imports: HashMap<String, (String, String)>, // alias/method_name -> (type_name, method_name)
    local_functions: HashSet<String>,           // Set of locally defined function names
    local_functions_with_bump: HashSet<String>, // Functions that need bump parameter
    data_classes_with_lifetime: HashSet<String>, // Track data classes that need lifetime parameters
    data_classes: HashSet<String>,              // Track all data classes
    generating_bump_function: bool, // Track when generating function with bump parameter
    config: Config,
}

impl CodeGenerator {
    pub fn with_config(config: Config) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            imports: HashMap::new(),
            local_functions: HashSet::new(),
            local_functions_with_bump: HashSet::new(),
            data_classes_with_lifetime: HashSet::new(),
            data_classes: HashSet::new(),
            generating_bump_function: false,
            config,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // First pass: collect all locally defined function names and data classes with lifetimes
        for stmt in &program.statements {
            match stmt {
                Stmt::FunDecl(fun_decl) => {
                    self.local_functions.insert(fun_decl.name.clone());
                    if fun_decl.has_hidden_bump {
                        self.local_functions_with_bump.insert(fun_decl.name.clone());
                    }
                }
                Stmt::DataClass(data_class) => {
                    // Track all data classes
                    self.data_classes.insert(data_class.name.clone());

                    // Check if this data class needs lifetime parameters
                    let needs_lifetime = data_class.fields.iter().any(|field| {
                        matches!(
                            field.field_type.base,
                            BaseType::Str | BaseType::String | BaseType::Custom(_)
                        ) || field.field_type.reference_depth > 0
                    });
                    if needs_lifetime {
                        self.data_classes_with_lifetime
                            .insert(data_class.name.clone());
                    }
                }
                _ => {}
            }
        }

        // Skip bumpalo import - use fully qualified names instead

        // Second pass: generate code
        for stmt in &program.statements {
            self.generate_statement(stmt);
        }
        self.output.clone()
    }

    fn generate_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr, inline_comment) => {
                self.indent();

                // Check if this is a method call with its own comment
                let method_comment = if let Expr::MethodCall(method_call) = expr {
                    method_call.inline_comment.clone()
                } else {
                    None
                };

                self.generate_expression(expr);
                self.output.push(';');

                // Generate method comment after semicolon, or statement comment if no method comment
                if let Some(comment) = method_comment {
                    self.generate_inline_comment(&Some(comment));
                } else {
                    self.generate_inline_comment(inline_comment);
                }

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
            Stmt::Import(import) => {
                // Track the import for later use
                let key = import
                    .alias
                    .clone()
                    .unwrap_or_else(|| import.method_name.clone());
                self.imports
                    .insert(key, (import.type_name.clone(), import.method_name.clone()));
                // Don't generate any Rust code for imports
            }
            Stmt::DataClass(data_class) => {
                self.generate_data_class(data_class);
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

            // Check if initializer is a method call with its own comment
            let method_comment = if let Expr::MethodCall(method_call) = initializer {
                method_call.inline_comment.clone()
            } else {
                None
            };

            self.generate_expression(initializer);
            self.output.push(';');

            // Generate method comment after semicolon, or variable comment if no method comment
            if let Some(comment) = method_comment {
                self.generate_inline_comment(&Some(comment));
            } else {
                self.generate_inline_comment(inline_comment);
            }
        } else {
            self.output.push(';');
            self.generate_inline_comment(inline_comment);
        }

        self.output.push('\n');
    }

    fn generate_function_declaration(&mut self, fun_decl: &FunDeclStmt) {
        self.indent();
        self.output.push_str("fn ");
        let snake_name = self.camel_to_snake_case(&fun_decl.name);
        self.output.push_str(&snake_name);

        // Add lifetime parameter if this function has a hidden bump parameter
        if fun_decl.has_hidden_bump {
            self.output.push_str("<'a>");
            self.generating_bump_function = true;
        }

        self.output.push('(');
        self.generate_comma_separated_params(&fun_decl.params, fun_decl.has_hidden_bump);
        self.output.push(')');

        if let Some(return_type) = &fun_decl.return_type {
            self.output.push_str(" -> ");
            self.generate_type(return_type);
        }

        self.output.push(' ');

        // Special handling for main function: only initialize bump allocator if needed
        if fun_decl.name == "main" {
            self.output.push_str("{\n");
            self.indent_level += 1;

            // Check if bump allocation is actually used in the main function body
            let needs_bump = self.uses_bump_allocation(&fun_decl.body);
            if needs_bump {
                self.indent();
                self.output.push_str("let bump = &bumpalo::Bump::new();\n");
            }

            // Generate the body content but skip the outer braces since we're handling them
            if let Stmt::Block(statements) = fun_decl.body.as_ref() {
                for stmt in statements {
                    self.generate_statement(stmt);
                }
            } else {
                self.generate_statement(&fun_decl.body);
            }

            self.indent_level -= 1;
            self.indent();
            self.output.push_str("}\n");
        } else {
            self.generate_statement(&fun_decl.body);
        }

        // Reset bump function flag
        self.generating_bump_function = false;
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

    fn generate_data_class(&mut self, data_class: &DataClassStmt) {
        // Check if any fields are reference types
        let needs_lifetime = data_class.fields.iter().any(|field| {
            matches!(
                field.field_type.base,
                BaseType::Str | BaseType::String | BaseType::Custom(_)
            ) || field.field_type.reference_depth > 0
        });

        self.indent();
        self.output.push_str("#[derive(Debug, Clone)]\n");
        self.indent();
        self.output.push_str("pub struct ");
        self.output.push_str(&data_class.name);

        if needs_lifetime {
            self.output.push_str("<'a>");
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;

        // Generate fields
        for field in &data_class.fields {
            self.indent();
            self.output.push_str("pub ");
            self.output.push_str(&self.camel_to_snake_case(&field.name));
            self.output.push_str(": ");

            // Generate the field type with lifetime if needed
            if needs_lifetime && field.field_type.reference_depth > 0 {
                // For reference types that need lifetime annotations
                for _ in 0..field.field_type.reference_depth {
                    self.output.push_str("&'a ");
                }
                // Check if the base type is a custom type that needs lifetime
                if let BaseType::Custom(name) = &field.field_type.base {
                    self.output.push_str(name);
                    if self.data_classes_with_lifetime.contains(name) {
                        self.output.push_str("<'a>");
                    }
                } else {
                    self.generate_base_type(&field.field_type.base);
                }
            } else {
                self.generate_type(&field.field_type);
            }
            self.output.push_str(",\n");
        }

        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}\n\n");
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
            Expr::Unary(unary) => {
                match &unary.operator {
                    UnaryOp::Minus => {
                        self.output.push('-');
                        // Wrap non-simple expressions in parentheses
                        match unary.operand.as_ref() {
                            Expr::Literal(_) | Expr::Identifier(_) => {
                                self.generate_expression(&unary.operand);
                            }
                            Expr::Unary(_) => {
                                // Wrap nested unary to avoid -- (double negation)
                                self.output.push('(');
                                self.generate_expression(&unary.operand);
                                self.output.push(')');
                            }
                            _ => {
                                self.output.push('(');
                                self.generate_expression(&unary.operand);
                                self.output.push(')');
                            }
                        }
                    }
                }
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
            Expr::FieldAccess(field_access) => {
                self.generate_field_access(field_access);
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
            LiteralExpr::Unit => {
                self.output.push_str("()");
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
            if self.generating_bump_function {
                self.output.push_str("&'a ");
            } else {
                self.output.push('&');
            }
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
            BaseType::Custom(name) => {
                self.output.push_str(name);
                // Add lifetime parameter for custom types in bump functions if they need lifetime
                if self.generating_bump_function && self.data_classes_with_lifetime.contains(name) {
                    self.output.push_str("<'a>");
                }
            }
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

    fn generate_comma_separated_params(&mut self, params: &[Parameter], include_bump: bool) {
        let mut first = true;

        if include_bump {
            self.output.push_str("bump: &'a bumpalo::Bump");
            first = false;
        }

        for param in params {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            let snake_name = self.camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type);
            self.generate_inline_comment_as_block(&param.inline_comment);
        }
    }

    fn generate_comma_separated_args(&mut self, args: &[Argument]) {
        let mut first = true;
        for arg in args {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            match arg {
                Argument::Bare(expr, comment) => {
                    self.generate_expression(expr);
                    self.generate_inline_comment(comment);
                }
                Argument::Named(name, expr, comment) => {
                    self.output.push_str(&self.camel_to_snake_case(name));
                    self.output.push_str(": ");
                    self.generate_expression(expr);
                    self.generate_inline_comment(comment);
                }
            }
        }
    }

    fn generate_comma_separated_args_for_function_call_with_multiline(
        &mut self,
        args: &[Argument],
        is_multiline: bool,
    ) {
        if is_multiline && !args.is_empty() {
            // Generate multiline format
            self.output.push('\n');
            for (i, arg) in args.iter().enumerate() {
                self.indent_level += 1;
                self.indent();

                match arg {
                    Argument::Bare(expr, comment) => {
                        self.generate_expression(expr);
                        self.generate_inline_comment_as_block(comment);
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        self.generate_expression(expr);
                        self.generate_inline_comment_as_block(comment);
                    }
                }

                if i < args.len() - 1 {
                    self.output.push(',');
                }
                self.output.push('\n');
                self.indent_level -= 1;
            }
            self.indent();
        } else {
            // Generate single line format
            let mut first = true;
            for arg in args {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                match arg {
                    Argument::Bare(expr, comment) => {
                        self.generate_expression(expr);
                        self.generate_inline_comment_as_block(comment);
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        self.generate_expression(expr);
                        self.generate_inline_comment_as_block(comment);
                    }
                }
            }
        }
    }

    fn generate_comma_separated_exprs(&mut self, exprs: &[Expr]) {
        let mut first = true;
        for expr in exprs {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            self.generate_expression(expr);
        }
    }

    fn generate_generic_call(&mut self, call: &CallExpr) {
        self.generate_expression(&call.callee);
        self.output.push('(');
        self.generate_comma_separated_args_for_function_call_with_multiline(
            &call.args,
            call.is_multiline,
        );
        self.output.push(')');
    }

    fn generate_call_expression(&mut self, call: &CallExpr) {
        if let Expr::Identifier(name) = call.callee.as_ref() {
            // Check if this is a data class constructor
            if self.data_classes.contains(name) {
                // This is struct initialization (works with positional, named, or mixed arguments)
                self.output.push_str(name);
                self.output.push_str(" { ");
                self.generate_comma_separated_args(&call.args);
                self.output.push_str(" }");
                return;
            }

            if name == "MutRef" {
                // Special case: MutRef(value) becomes &mut (&value).clone()
                if call.args.len() != 1 {
                    panic!(
                        "MutRef() requires exactly one argument, found {}",
                        call.args.len()
                    );
                }
                self.output.push_str("&mut (&");
                if let Argument::Bare(expr, _) = &call.args[0] {
                    self.generate_expression(expr);
                } else {
                    panic!("MutRef() does not support named arguments");
                }
                self.output.push_str(").clone()");
            } else if self.local_functions.contains(name) {
                // Locally defined function: regular call with snake_case conversion
                let snake_name = self.camel_to_snake_case(name);
                self.output.push_str(&snake_name);
                self.output.push('(');

                // If this function has hidden bump, add bump as first argument
                if self.local_functions_with_bump.contains(name) {
                    self.output.push_str("bump");
                    if !call.args.is_empty() {
                        self.output.push_str(", ");
                    }
                }

                self.generate_comma_separated_args_for_function_call_with_multiline(
                    &call.args,
                    call.is_multiline,
                );
                self.output.push(')');
            } else if let Some((type_name, original_method)) = self.imports.get(name) {
                // Imported function/constructor: use UFCS
                let snake_method = self.camel_to_snake_case(original_method);
                self.output.push_str(type_name);
                self.output.push_str("::");
                self.output.push_str(&snake_method);
                self.output.push('(');
                self.generate_comma_separated_args_for_function_call_with_multiline(
                    &call.args,
                    call.is_multiline,
                );
                self.output.push(')');
            } else if self.is_rust_macro(name) {
                self.output.push_str(name);
                self.output.push('!');
                self.output.push('(');
                self.generate_comma_separated_args_for_function_call_with_multiline(
                    &call.args,
                    call.is_multiline,
                );
                self.output.push(')');
            } else {
                // Default case for identifiers that aren't special
                self.generate_generic_call(call);
            }
        } else {
            self.generate_generic_call(call);
        }
    }

    fn generate_method_call_expression(&mut self, method_call: &MethodCallExpr) {
        if let Some((type_name, original_method)) = self.imports.get(&method_call.method) {
            // Imported method: use UFCS (explicit imports have highest priority)
            let snake_method = self.camel_to_snake_case(original_method);
            self.output.push_str(type_name);
            self.output.push_str("::");
            self.output.push_str(&snake_method);
            self.output.push('(');

            // First argument is the object
            self.generate_expression(&method_call.object);

            // Then the rest of the arguments
            for arg in &method_call.args {
                self.output.push_str(", ");
                self.generate_expression(arg);
            }
            self.output.push(')');
        } else if method_call.method == "ref" && method_call.args.is_empty() {
            // Special case: ownedValue.ref() becomes &ownedValue
            // This converts Own<T> to T (which is &T in Rust)
            self.output.push('&');
            self.generate_expression(&method_call.object);
        } else if method_call.method == "bumpRef" && method_call.args.is_empty() {
            // Special case: value.bumpRef() becomes bump.alloc(value)
            // This moves the value to bump allocation
            self.output.push_str("bump.alloc(");
            self.generate_expression(&method_call.object);
            self.output.push(')');
        } else if method_call.method == "mutRef" && method_call.args.is_empty() {
            // Special case: obj.mutRef() becomes &mut obj
            // No automatic cloning - users must explicitly call .clone() if needed
            self.output.push_str("&mut ");
            self.generate_expression(&method_call.object);
        } else if method_call.method == "clone" && method_call.args.is_empty() {
            // Special case: obj.clone() becomes Clone::clone(obj) using UFCS
            // This avoids auto-ref and makes borrowing explicit
            self.output.push_str("Clone::clone(");
            self.generate_expression(&method_call.object);
            self.output.push(')');
        } else if method_call.method == "toString" && method_call.args.is_empty() {
            // Special case: obj.toString() becomes ToString::to_string(obj) using UFCS
            // This is pre-imported like clone
            self.output.push_str("ToString::to_string(");
            self.generate_expression(&method_call.object);
            self.output.push(')');
        } else {
            // Regular method call: obj.method(args)
            let snake_method = self.camel_to_snake_case(&method_call.method);
            self.generate_expression(&method_call.object);
            self.output.push('.');
            self.output.push_str(&snake_method);
            self.output.push('(');
            self.generate_comma_separated_exprs(&method_call.args);
            self.output.push(')');
        }

        // Note: Method call comments are now handled by the statement generator to ensure proper placement after semicolons
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

                // Check if this is a block comment (starts with /*) or line comment
                if content.starts_with("/*") {
                    // Block comment - output as-is
                    self.output.push_str(content);
                } else {
                    // Line comment - add // prefix
                    self.output.push_str("//");
                    self.output.push_str(content);
                }
            }
        }
    }

    fn generate_inline_comment_as_block(&mut self, inline_comment: &Option<(String, String)>) {
        if let Some((content, whitespace)) = inline_comment {
            if self.config.preserve_comments {
                self.output.push_str(whitespace);

                // Check if this is a block comment (starts with /*) or line comment
                if content.starts_with("/*") {
                    // Already a block comment - output as-is
                    self.output.push_str(content);
                } else {
                    // Line comment - convert to block comment for better syntax compatibility
                    self.output.push_str("/*");
                    self.output.push_str(content);
                    self.output.push_str("*/");
                }
            }
        }
    }

    fn generate_field_access(&mut self, field_access: &FieldAccessExpr) {
        self.generate_expression(&field_access.object);
        self.output.push('.');
        self.output
            .push_str(&self.camel_to_snake_case(&field_access.field));
    }

    fn uses_bump_allocation(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expression(expr, _) => self.expr_uses_bump_allocation(expr),
            Stmt::VarDecl(var_decl, _) => {
                if let Some(initializer) = &var_decl.initializer {
                    self.expr_uses_bump_allocation(initializer)
                } else {
                    false
                }
            }
            Stmt::FunDecl(_) => false, // Function declarations don't use bump directly
            Stmt::If(if_stmt) => {
                self.expr_uses_bump_allocation(&if_stmt.condition)
                    || self.uses_bump_allocation(&if_stmt.then_branch)
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |stmt| self.uses_bump_allocation(stmt))
            }
            Stmt::While(while_stmt) => {
                self.expr_uses_bump_allocation(&while_stmt.condition)
                    || self.uses_bump_allocation(&while_stmt.body)
            }
            Stmt::Return(expr, _) => expr
                .as_ref()
                .map_or(false, |e| self.expr_uses_bump_allocation(e)),
            Stmt::Block(statements) => statements
                .iter()
                .any(|stmt| self.uses_bump_allocation(stmt)),
            Stmt::Comment(_) | Stmt::Import(_) | Stmt::DataClass(_) => false,
        }
    }

    fn expr_uses_bump_allocation(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) | Expr::Identifier(_) => false,
            Expr::Unary(unary) => self.expr_uses_bump_allocation(&unary.operand),
            Expr::Binary(binary) => {
                self.expr_uses_bump_allocation(&binary.left)
                    || self.expr_uses_bump_allocation(&binary.right)
            }
            Expr::Call(call) => {
                // Check if this is a call to a function that requires bump
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if self.local_functions_with_bump.contains(name) {
                        return true;
                    }
                }

                self.expr_uses_bump_allocation(&call.callee)
                    || call.args.iter().any(|arg| match arg {
                        Argument::Bare(expr, _) => self.expr_uses_bump_allocation(expr),
                        Argument::Named(_, expr, _) => self.expr_uses_bump_allocation(expr),
                    })
            }
            Expr::MethodCall(method_call) => {
                // Check if this is a .bumpRef() method call
                if method_call.method == "bumpRef" && method_call.args.is_empty() {
                    return true;
                }
                // Also check the object and arguments
                self.expr_uses_bump_allocation(&method_call.object)
                    || method_call
                        .args
                        .iter()
                        .any(|arg| self.expr_uses_bump_allocation(arg))
            }
            Expr::FieldAccess(field_access) => self.expr_uses_bump_allocation(&field_access.object),
        }
    }
}
