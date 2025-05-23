use crate::ast::*;

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
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
            Stmt::Expression(expr) => {
                self.indent();
                self.generate_expression(expr);
                self.output.push_str(";\n");
            }
            Stmt::VarDecl(var_decl) => {
                self.generate_var_declaration(var_decl);
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
            Stmt::Return(expr) => {
                self.indent();
                self.output.push_str("return");
                if let Some(expr) = expr {
                    self.output.push(' ');
                    self.generate_expression(expr);
                }
                self.output.push_str(";\n");
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
        }
    }

    fn generate_var_declaration(&mut self, var_decl: &VarDeclStmt) {
        self.indent();

        if var_decl.is_mutable {
            self.output.push_str("let mut ");
        } else {
            self.output.push_str("let ");
        }

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

        self.output.push_str(";\n");
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
        self.output.push_str("while ");
        self.generate_expression(&while_stmt.condition);
        self.output.push(' ');
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
        match type_annotation {
            Type::Int => self.output.push_str("i64"),
            Type::Str => self.output.push_str("str"),
            Type::String => self.output.push_str("String"),
            Type::Bool => self.output.push_str("bool"),
            Type::Unit => self.output.push_str("()"),
            Type::Nothing => self.output.push_str("!"),
            Type::Ref(inner) => {
                self.output.push('&');
                self.generate_type(inner);
            }
            Type::Box(inner) => {
                self.output.push_str("Box<");
                self.generate_type(inner);
                self.output.push('>');
            }
            Type::Custom(name) => self.output.push_str(name),
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

    fn camel_to_snake_case(&self, name: &str) -> String {
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
            if self.is_rust_macro(name) {
                self.output.push_str(name);
                self.output.push('!');
            } else {
                self.generate_expression(&call.callee);
            }
        } else {
            self.generate_expression(&call.callee);
        }
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.generate_expression(arg);
        }
        self.output.push(')');
    }

    fn generate_method_call_expression(&mut self, method_call: &MethodCallExpr) {
        if method_call.method == "ref" && method_call.args.is_empty() {
            // Special case: obj.ref() becomes &obj
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake_case() {
        let codegen = CodeGenerator::new();

        assert_eq!(codegen.camel_to_snake_case("camelCase"), "camel_case");
        assert_eq!(codegen.camel_to_snake_case("CamelCase"), "camel_case");
        assert_eq!(codegen.camel_to_snake_case("simpleVar"), "simple_var");
        assert_eq!(
            codegen.camel_to_snake_case("veryLongCamelCaseVariableName"),
            "very_long_camel_case_variable_name"
        );
        assert_eq!(codegen.camel_to_snake_case("a"), "a");
        assert_eq!(codegen.camel_to_snake_case("aB"), "a_b");
        assert_eq!(codegen.camel_to_snake_case("aBc"), "a_bc");
        assert_eq!(codegen.camel_to_snake_case("XMLParser"), "x_m_l_parser");
        assert_eq!(
            codegen.camel_to_snake_case("httpURLConnection"),
            "http_u_r_l_connection"
        );
        assert_eq!(codegen.camel_to_snake_case("main"), "main");
        assert_eq!(codegen.camel_to_snake_case("calculateSum"), "calculate_sum");
    }

    #[test]
    fn test_camel_case_transpilation() {
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        let source = r#"
fun calculateSum(firstNumber: Int, secondNumber: Int): Int {
    val totalResult: Int = firstNumber + secondNumber
    return totalResult
}
"#;

        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("Parse should succeed");

        let mut codegen = CodeGenerator::new();
        let rust_code = codegen.generate(&program);

        assert!(rust_code.contains("fn calculate_sum"));
        assert!(rust_code.contains("first_number: i64"));
        assert!(rust_code.contains("second_number: i64"));
        assert!(rust_code.contains("let total_result: i64"));
        assert!(rust_code.contains("first_number + second_number"));
        assert!(rust_code.contains("return total_result"));
    }

    #[test]
    fn test_readme_examples() {
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        use std::fs;

        let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");

        let examples = extract_code_examples(&readme_content);

        for (veltrano_code, expected_rust) in examples {
            let mut lexer = Lexer::new(veltrano_code.clone());
            let all_tokens = lexer.tokenize();
            let tokens = crate::lexer::skip_comments(all_tokens);
            let mut parser = Parser::new(tokens);

            if let Ok(program) = parser.parse() {
                let mut codegen = CodeGenerator::new();
                let actual_rust = codegen.generate(&program);

                // Normalize whitespace for comparison
                let actual_normalized = normalize_code(&actual_rust);
                let expected_normalized = normalize_code(&expected_rust);

                assert_eq!(
                    actual_normalized, expected_normalized,
                    "\nVeltrano code:\n{}\n\nExpected Rust:\n{}\n\nActual Rust:\n{}",
                    veltrano_code, expected_rust, actual_rust
                );
            }
        }
    }

    #[test]
    fn test_readme_rust_outputs_compile() {
        use std::fs;
        use std::process::Command;

        let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");
        let rust_examples = extract_rust_code_examples(&readme_content);

        for (index, rust_code) in rust_examples.iter().enumerate() {
            // Create a temporary Rust file for each example
            let temp_file = format!("/tmp/readme_example_{}.rs", index);

            // Wrap the code in a main function if it's not already a complete program
            let complete_rust_code = if rust_code.contains("fn main") {
                rust_code.clone()
            } else {
                format!("fn main() {{\n{}\n}}", rust_code)
            };

            fs::write(&temp_file, &complete_rust_code)
                .expect(&format!("Failed to write temp file {}", temp_file));

            // Try to compile the Rust code
            let output = Command::new("rustc")
                .arg("--crate-type")
                .arg("bin")
                .arg("--edition")
                .arg("2021")
                .arg("-o")
                .arg(&format!("/tmp/readme_example_{}", index))
                .arg(&temp_file)
                .output()
                .expect("Failed to execute rustc");

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "README Rust example {} failed to compile:\n{}\n\nCode:\n{}",
                    index, stderr, complete_rust_code
                );
            }

            // Clean up temporary files
            let _ = fs::remove_file(&temp_file);
            let _ = fs::remove_file(&format!("/tmp/readme_example_{}", index));
        }
    }

    #[test]
    fn test_readme_veltrano_snippets_transpile_and_compile() {
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        use std::fs;
        use std::process::Command;

        let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");
        let veltrano_examples = extract_veltrano_code_examples(&readme_content);

        for (index, veltrano_code) in veltrano_examples.iter().enumerate() {
            // Try to transpile the Veltrano code
            let mut lexer = Lexer::new(veltrano_code.clone());
            let all_tokens = lexer.tokenize();
            let tokens = crate::lexer::skip_comments(all_tokens);
            let mut parser = Parser::new(tokens);

            let program = match parser.parse() {
                Ok(program) => program,
                Err(err) => {
                    panic!(
                        "README Veltrano example {} failed to parse:\n{}\n\nCode:\n{}",
                        index, err, veltrano_code
                    );
                }
            };

            // Generate Rust code
            let mut codegen = CodeGenerator::new();
            let rust_code = codegen.generate(&program);

            // Create a temporary Rust file
            let temp_file = format!("/tmp/readme_veltrano_example_{}.rs", index);

            // Wrap the code in a main function if it's not already a complete program
            let complete_rust_code = if rust_code.contains("fn main") {
                rust_code.clone()
            } else {
                format!("fn main() {{\n{}\n}}", rust_code)
            };

            fs::write(&temp_file, &complete_rust_code)
                .expect(&format!("Failed to write temp file {}", temp_file));

            // Try to compile the generated Rust code
            let output = Command::new("rustc")
                .arg("--crate-type")
                .arg("bin")
                .arg("--edition")
                .arg("2021")
                .arg("-o")
                .arg(&format!("/tmp/readme_veltrano_example_{}", index))
                .arg(&temp_file)
                .output()
                .expect("Failed to execute rustc");

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "README Veltrano example {} transpiled but failed to compile:\n{}\n\nVeltrano code:\n{}\n\nGenerated Rust code:\n{}",
                    index, stderr, veltrano_code, complete_rust_code
                );
            }

            // Clean up temporary files
            let _ = fs::remove_file(&temp_file);
            let _ = fs::remove_file(&format!("/tmp/readme_veltrano_example_{}", index));
        }
    }

    fn extract_code_examples(readme: &str) -> Vec<(String, String)> {
        let mut examples = Vec::new();
        let lines: Vec<&str> = readme.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            // Look for "Transpiles to:" followed by rust code
            if lines[i].contains("**Transpiles to:**") {
                // Look backwards for the most recent kotlin block
                let mut kotlin_start = None;
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    if lines[j].trim() == "```kotlin" {
                        kotlin_start = Some(j);
                        break;
                    }
                    // Stop if we hit another "Transpiles to:" or similar
                    if lines[j].contains("**Transpiles to:**") || lines[j].contains("**Examples:**")
                    {
                        break;
                    }
                }

                if let Some(kotlin_start_idx) = kotlin_start {
                    // Extract kotlin code
                    let mut veltrano_code = String::new();
                    let mut k = kotlin_start_idx + 1;
                    while k < lines.len() && lines[k].trim() != "```" {
                        veltrano_code.push_str(lines[k]);
                        veltrano_code.push('\n');
                        k += 1;
                    }

                    // Look forward for rust code after "Transpiles to:"
                    while i < lines.len() && lines[i].trim() != "```rust" {
                        i += 1;
                    }

                    if i < lines.len() && lines[i].trim() == "```rust" {
                        let mut rust_code = String::new();
                        i += 1;

                        while i < lines.len() && lines[i].trim() != "```" {
                            rust_code.push_str(lines[i]);
                            rust_code.push('\n');
                            i += 1;
                        }

                        if !veltrano_code.trim().is_empty() && !rust_code.trim().is_empty() {
                            examples.push((
                                veltrano_code.trim().to_string(),
                                rust_code.trim().to_string(),
                            ));
                        }
                    }
                }
            }
            i += 1;
        }

        examples
    }

    fn extract_rust_code_examples(readme: &str) -> Vec<String> {
        let mut examples = Vec::new();
        let lines: Vec<&str> = readme.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            if lines[i].trim() == "```rust" {
                let mut rust_code = String::new();
                i += 1;

                while i < lines.len() && lines[i].trim() != "```" {
                    rust_code.push_str(lines[i]);
                    rust_code.push('\n');
                    i += 1;
                }

                if !rust_code.trim().is_empty() {
                    examples.push(rust_code.trim().to_string());
                }
            }
            i += 1;
        }

        examples
    }

    fn extract_veltrano_code_examples(readme: &str) -> Vec<String> {
        let mut examples = Vec::new();
        let lines: Vec<&str> = readme.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            if lines[i].trim() == "```kotlin" {
                let mut veltrano_code = String::new();
                i += 1;

                while i < lines.len() && lines[i].trim() != "```" {
                    veltrano_code.push_str(lines[i]);
                    veltrano_code.push('\n');
                    i += 1;
                }

                if !veltrano_code.trim().is_empty() {
                    // Check if this Veltrano snippet has a corresponding output
                    let mut has_rust_output = false;
                    let mut j = i;

                    // Look ahead for "**Transpiles to:**" or "**Output" patterns
                    while j < lines.len() && j < i + 10 {
                        if lines[j].contains("**Transpiles to:**") || lines[j].contains("**Output")
                        {
                            has_rust_output = true;
                            break;
                        }
                        // Stop looking if we hit another code block or major section
                        if lines[j].trim().starts_with("```") || lines[j].starts_with("##") {
                            break;
                        }
                        j += 1;
                    }

                    // Only include snippets that don't have a specified output
                    if !has_rust_output {
                        examples.push(veltrano_code.trim().to_string());
                    }
                }
            }
            i += 1;
        }

        examples
    }

    fn normalize_code(code: &str) -> String {
        code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
