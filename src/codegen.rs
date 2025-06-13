use crate::ast::*;
use crate::config::Config;
use crate::rust_interop::camel_to_snake_case;
use crate::rust_interop::RustInteropRegistry;
use crate::type_checker::MethodResolution;
use crate::types::VeltranoType;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Errors that can occur during code generation
#[derive(Debug)]
pub enum CodegenError {
    /// Invalid syntax when calling a data class constructor
    InvalidDataClassSyntax { constructor: String, reason: String },
    /// Shorthand syntax used in wrong context
    InvalidShorthandUsage { field_name: String, context: String },
    /// Invalid arguments for built-in functions
    InvalidBuiltinArguments { builtin: String, reason: String },
    /// Method requires import but wasn't imported
    MissingImport { method: String, type_name: String },
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::InvalidDataClassSyntax {
                constructor,
                reason,
            } => {
                write!(
                    f,
                    "Invalid syntax for data class '{}': {}",
                    constructor, reason
                )
            }
            CodegenError::InvalidShorthandUsage {
                field_name,
                context,
            } => {
                write!(
                    f,
                    "Shorthand syntax (.{}) is only valid for data class constructors, not {}",
                    field_name, context
                )
            }
            CodegenError::InvalidBuiltinArguments { builtin, reason } => {
                write!(f, "Invalid arguments for {}: {}", builtin, reason)
            }
            CodegenError::MissingImport { method, type_name } => {
                write!(f, "Method '{}' requires an explicit import. Add 'import {}.{}' at the top of your file.", method, type_name, method)
            }
        }
    }
}

impl std::error::Error for CodegenError {}

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    imports: HashMap<String, (String, String)>, // alias/method_name -> (type_name, method_name)
    local_functions: HashSet<String>,           // Set of locally defined function names
    local_functions_with_bump: HashSet<String>, // Functions that need bump parameter
    data_classes_with_lifetime: HashSet<String>, // Track data classes that need lifetime parameters
    data_classes: HashSet<String>,              // Track all data classes
    generating_bump_function: bool, // Track when generating function with bump parameter
    trait_checker: RustInteropRegistry, // For trait-based type checking
    config: Config,
    method_resolutions: HashMap<usize, MethodResolution>, // Method call ID -> resolved import
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
            trait_checker: RustInteropRegistry::new(),
            config,
            method_resolutions: HashMap::new(),
        }
    }

    /// Set method resolutions from the type checker
    pub fn set_method_resolutions(&mut self, resolutions: HashMap<usize, MethodResolution>) {
        self.method_resolutions = resolutions;
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, CodegenError> {
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
                    let needs_lifetime = data_class
                        .fields
                        .iter()
                        .any(|field| self.type_needs_lifetime(&field.field_type));
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
        Ok(self.output.clone())
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

        let snake_name = camel_to_snake_case(&var_decl.name);
        self.output.push_str(&snake_name);

        if let Some(type_annotation) = &var_decl.type_annotation {
            self.output.push_str(": ");
            self.generate_type(type_annotation);
        }

        if let Some(initializer) = &var_decl.initializer {
            self.output.push_str(" = ");

            // Collect all comments from method chain if this is a method call
            let method_chain_comments = self.collect_method_chain_comments(initializer);

            self.generate_expression(initializer);
            self.output.push(';');

            // Generate all method chain comments after semicolon
            if !method_chain_comments.is_empty() {
                // Output each comment in its original style
                for (i, comment) in method_chain_comments.iter().enumerate() {
                    if i == 0 {
                        // First comment uses its original whitespace
                        self.generate_inline_comment(&Some(comment.clone()));
                    } else {
                        // Subsequent comments get minimal whitespace to stay on same line
                        let (content, _) = comment;
                        self.generate_inline_comment(&Some((content.clone(), " ".to_string())));
                    }
                }
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
        let snake_name = camel_to_snake_case(&fun_decl.name);
        self.output.push_str(&snake_name);

        // Add lifetime parameter if this function has a hidden bump parameter
        if fun_decl.has_hidden_bump {
            self.output.push_str("<'a>");
            self.generating_bump_function = true;
        }

        self.output.push('(');

        // Check if we should use multiline formatting for parameters
        let use_multiline = fun_decl.params.iter().any(|p| p.inline_comment.is_some());

        if use_multiline && !fun_decl.params.is_empty() {
            self.generate_multiline_params(&fun_decl.params, fun_decl.has_hidden_bump);
        } else {
            self.generate_comma_separated_params(&fun_decl.params, fun_decl.has_hidden_bump);
        }

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
        let needs_lifetime = data_class
            .fields
            .iter()
            .any(|field| self.type_needs_lifetime(&field.field_type));

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
            self.output.push_str(&camel_to_snake_case(&field.name));
            self.output.push_str(": ");

            // Generate the field type with lifetime if needed
            if needs_lifetime {
                self.generate_data_class_field_type(&field.field_type);
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
                let snake_name = camel_to_snake_case(name);
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
                if let Err(e) = self.generate_call_expression(call) {
                    panic!("Code generation error: {}", e);
                }
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

    fn generate_type(&mut self, type_annotation: &VeltranoType) {
        use crate::types::TypeConstructor;

        // For data class types that need lifetime parameters, we need special handling
        if let TypeConstructor::Custom(name) = &type_annotation.constructor {
            if self.data_classes_with_lifetime.contains(name) {
                // For naturally referenced custom types with lifetime parameters
                self.output.push('&');
                if self.generating_bump_function {
                    self.output.push_str("'a ");
                }
                self.output.push_str(name);
                if self.generating_bump_function {
                    self.output.push_str("<'a>");
                }
                return;
            }
        }

        // Use the new to_rust_type_with_lifetime method
        let lifetime = if self.generating_bump_function {
            Some("'a".to_string())
        } else {
            None
        };

        let rust_type =
            type_annotation.to_rust_type_with_lifetime(&mut self.trait_checker, lifetime);
        self.output.push_str(&rust_type.to_rust_syntax());
    }

    /// Check if a type needs lifetime parameters (is naturally referenced)
    fn type_needs_lifetime(&mut self, veltrano_type: &VeltranoType) -> bool {
        use crate::types::TypeConstructor;

        match &veltrano_type.constructor {
            // Reference types always need lifetimes
            TypeConstructor::Ref | TypeConstructor::MutRef => true,
            // Use trait checking for base types
            _ if veltrano_type.args.is_empty() => {
                !veltrano_type.implements_copy(&mut self.trait_checker)
            }
            // Composed types need further analysis
            _ => false,
        }
    }

    fn generate_data_class_field_type(&mut self, type_annotation: &VeltranoType) {
        use crate::types::TypeConstructor;

        // For data class fields, we always need lifetime 'a for reference types
        // Special handling for custom types with lifetime parameters
        if let TypeConstructor::Custom(name) = &type_annotation.constructor {
            if self.data_classes_with_lifetime.contains(name) {
                self.output.push_str("&'a ");
                self.output.push_str(name);
                self.output.push_str("<'a>");
                return;
            }
        }

        // For owned custom types in data class fields
        if let TypeConstructor::Own = &type_annotation.constructor {
            if let Some(inner) = type_annotation.inner() {
                if let TypeConstructor::Custom(name) = &inner.constructor {
                    self.output.push_str(name);
                    if self.data_classes_with_lifetime.contains(name) {
                        self.output.push_str("<'a>");
                    }
                    return;
                }
            }
        }

        // For all other types, use the standard conversion with lifetime 'a
        let rust_type = type_annotation
            .to_rust_type_with_lifetime(&mut self.trait_checker, Some("'a".to_string()));
        self.output.push_str(&rust_type.to_rust_syntax());
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
            let snake_name = camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type);
            self.generate_inline_comment_as_block(&param.inline_comment);
        }
    }

    fn generate_multiline_params(&mut self, params: &[Parameter], include_bump: bool) {
        self.output.push('\n');
        self.indent_level += 1;

        if include_bump {
            self.indent();
            self.output.push_str("bump: &'a bumpalo::Bump");
            if !params.is_empty() {
                self.output.push(',');
            }
            self.output.push('\n');
        }

        for (i, param) in params.iter().enumerate() {
            self.indent();
            let snake_name = camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type);

            // Add comma if not the last parameter
            if i < params.len() - 1 {
                self.output.push(',');
            }

            // Generate inline comment as line comment, not block comment
            self.generate_inline_comment(&param.inline_comment);
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.indent();
    }

    fn generate_comma_separated_args_for_struct_init(&mut self, args: &[Argument]) {
        let mut first = true;
        for arg in args {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            match arg {
                Argument::Bare(_, _) => {
                    panic!("Data class constructors don't support positional arguments. Use named arguments or .field shorthand syntax");
                }
                Argument::Named(name, expr, comment) => {
                    self.output.push_str(&camel_to_snake_case(name));
                    self.output.push_str(": ");
                    self.generate_expression(expr);
                    self.generate_inline_comment(comment);
                }
                Argument::Shorthand(field_name, comment) => {
                    // Shorthand: generate field_name (variable matches field name)
                    self.output.push_str(&camel_to_snake_case(field_name));
                    self.generate_inline_comment(comment);
                }
                Argument::StandaloneComment(_, _) => {
                    // For single-line struct initialization, ignore standalone comments
                    first = true; // Don't add comma before next real argument
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
                        if i < args.len() - 1 {
                            self.output.push(',');
                        }
                        self.generate_inline_comment(comment);
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        self.generate_expression(expr);
                        if i < args.len() - 1 {
                            self.output.push(',');
                        }
                        self.generate_inline_comment(comment);
                    }
                    Argument::Shorthand(field_name, _) => {
                        panic!("Shorthand syntax (.{}) is only valid for data class constructors, not function calls", field_name);
                    }
                    Argument::StandaloneComment(content, whitespace) => {
                        // Generate standalone comment as its own line
                        if self.config.preserve_comments {
                            // Following the pattern from generate_comment for regular statements:
                            // The loop already called indent() to add base indentation.
                            // Now add any extra whitespace preserved by the lexer.
                            self.output.push_str(whitespace);
                            if content.starts_with("/*") {
                                // Block comment
                                self.output.push_str(content);
                            } else {
                                // Line comment
                                self.output.push_str("//");
                                self.output.push_str(content);
                            }
                        }
                        // Note: No comma or expression - this is just a comment line
                    }
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
                    Argument::Shorthand(field_name, _) => {
                        panic!("Shorthand syntax (.{}) is only valid for data class constructors, not function calls", field_name);
                    }
                    Argument::StandaloneComment(_, _) => {
                        // For single-line calls, standalone comments force multiline format
                        // This should not happen in practice as standalone comments should trigger multiline
                        // But handle it gracefully by ignoring the comment in single-line format
                        first = true; // Don't add comma before next real argument
                    }
                }
            }
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

    fn generate_call_expression(&mut self, call: &CallExpr) -> Result<(), CodegenError> {
        // First check if we have a type-checked resolution for this call (e.g., import alias)
        if let Some(resolution) = self.method_resolutions.get(&call.id) {
            // This is a resolved import alias (e.g., newVec -> Vec::new)
            let snake_method = camel_to_snake_case(&resolution.method_name);
            let type_name = resolution.rust_type.to_rust_syntax();

            self.output.push_str(&type_name);
            self.output.push_str("::");
            self.output.push_str(&snake_method);
            self.output.push('(');

            // Generate arguments
            self.generate_comma_separated_args_for_function_call_with_multiline(
                &call.args,
                call.is_multiline,
            );

            self.output.push(')');
            return Ok(());
        }

        if let Expr::Identifier(name) = call.callee.as_ref() {
            // Check if this is a data class constructor
            if self.data_classes.contains(name) {
                // This is struct initialization (works with positional, named, or mixed arguments)
                self.output.push_str(name);
                self.output.push_str(" { ");
                self.generate_comma_separated_args_for_struct_init(&call.args);
                self.output.push_str(" }");
                return Ok(());
            }

            if name == "MutRef" {
                // Special case: MutRef(value) becomes &mut (&value).clone()
                if call.args.len() != 1 {
                    return Err(CodegenError::InvalidBuiltinArguments {
                        builtin: "MutRef".to_string(),
                        reason: format!("requires exactly one argument, found {}", call.args.len()),
                    });
                }
                self.output.push_str("&mut (&");
                match &call.args[0] {
                    Argument::Bare(expr, _) => {
                        self.generate_expression(expr);
                    }
                    Argument::Shorthand(field_name, _) => {
                        // Shorthand behaves like Bare - just generate the identifier
                        let snake_name = camel_to_snake_case(field_name);
                        self.output.push_str(&snake_name);
                    }
                    Argument::Named(_, _, _) => {
                        return Err(CodegenError::InvalidBuiltinArguments {
                            builtin: "MutRef".to_string(),
                            reason: "does not support named arguments".to_string(),
                        });
                    }
                    Argument::StandaloneComment(_, _) => {
                        return Err(CodegenError::InvalidBuiltinArguments {
                            builtin: "MutRef".to_string(),
                            reason: "cannot have standalone comments as arguments".to_string(),
                        });
                    }
                }
                self.output.push_str(").clone()");
            } else if self.local_functions.contains(name) {
                // Locally defined function: regular call with snake_case conversion
                let snake_name = camel_to_snake_case(name);
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
                let snake_method = camel_to_snake_case(original_method);
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
        Ok(())
    }

    // Helper to collect all comments from a method chain
    fn collect_method_chain_comments(&self, expr: &Expr) -> Vec<(String, String)> {
        let mut comments = Vec::new();

        if let Expr::MethodCall(method_call) = expr {
            // First collect comments from the inner expression
            comments.extend(self.collect_method_chain_comments(&method_call.object));

            // Then add this method's comment if it exists
            if let Some(comment) = &method_call.inline_comment {
                comments.push(comment.clone());
            }
        }

        comments
    }

    fn generate_method_call_expression(&mut self, method_call: &MethodCallExpr) {
        // First check if we have a type-checked resolution for this method call
        crate::debug_println!(
            "DEBUG codegen: Looking for resolution for method call ID {}, method: {}",
            method_call.id,
            method_call.method
        );
        if let Some(resolution) = self.method_resolutions.get(&method_call.id) {
            // Use the resolved import
            crate::debug_println!(
                "DEBUG codegen: Found resolution - type: {:?}, method: {}",
                resolution.rust_type,
                resolution.method_name
            );
            let snake_method = camel_to_snake_case(&resolution.method_name);
            let type_name = resolution.rust_type.to_rust_syntax();

            self.output.push_str(&type_name);
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
        } else if let Some((type_name, original_method)) = self.imports.get(&method_call.method) {
            // Fallback to old behavior for non-type-checked code
            let snake_method = camel_to_snake_case(original_method);
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
            // Method requires import but wasn't imported
            panic!(
                "Method '{}' requires an explicit import. Add 'import {{Type}}.{}' at the top of your file.",
                method_call.method, method_call.method
            );
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
            .push_str(&camel_to_snake_case(&field_access.field));
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
                        Argument::Shorthand(_, _) => false, // Shorthand is just an identifier, doesn't use bump allocation
                        Argument::StandaloneComment(_, _) => false, // Comments don't use bump allocation
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
