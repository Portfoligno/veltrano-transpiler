#![allow(dead_code)]

pub mod snapshot_utils;

use std::fs;
use std::process::Command;
use veltrano::{
    ast::Program,
    codegen::CodeGenerator,
    config::Config,
    error::VeltranoError,
    lexer::Lexer,
    parser::Parser,
    type_checker::{TypeCheckError, VeltranoTypeChecker},
};

/// Test context that holds all common parameters for testing functions
#[derive(Clone)]
pub struct TestContext {
    pub config: Config,
    pub skip_type_check: bool,
    pub variable_injections: Vec<(String, String)>,
    pub test_name: String,
    pub expected_error: Option<String>,
    pub remove_error_lines: bool,
}

impl Default for TestContext {
    fn default() -> Self {
        TestContext {
            config: Config {
                preserve_comments: false,
            },
            skip_type_check: false,
            variable_injections: Vec::new(),
            test_name: "test".to_string(),
            expected_error: None,
            remove_error_lines: false,
        }
    }
}

impl TestContext {
    /// Create a new TestContext with the given config
    pub fn with_config(config: Config) -> Self {
        TestContext {
            config,
            ..Default::default()
        }
    }

    /// Set the test name
    pub fn with_name(mut self, name: &str) -> Self {
        self.test_name = name.to_string();
        self
    }

    /// Set skip_type_check flag
    pub fn skip_type_check(mut self, skip: bool) -> Self {
        self.skip_type_check = skip;
        self
    }

    /// Add a variable injection
    pub fn with_injection(mut self, var_name: &str, init_code: &str) -> Self {
        self.variable_injections
            .push((var_name.to_string(), init_code.to_string()));
        self
    }

    /// Set expected error message
    pub fn expect_error(mut self, error: &str) -> Self {
        self.expected_error = Some(error.to_string());
        self
    }

    /// Set remove_error_lines flag
    pub fn remove_error_lines(mut self, remove: bool) -> Self {
        self.remove_error_lines = remove;
        self
    }
}

/// Shared utility to parse Veltrano code into an AST
fn parse_veltrano_code(code: &str, config: Config) -> Result<Program, VeltranoError> {
    let mut lexer = Lexer::with_config(code.to_string(), config);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Generate Rust code from an AST program with optional method resolutions
fn generate_rust_code(
    program: &Program,
    config: Config,
    method_resolutions: Option<
        std::collections::HashMap<usize, veltrano::type_checker::MethodResolution>,
    >,
) -> String {
    let mut codegen = CodeGenerator::with_config(config);
    if let Some(resolutions) = method_resolutions {
        codegen.set_method_resolutions(resolutions);
    }
    codegen
        .generate(program)
        .expect("Code generation should succeed")
}

/// Shared utility to parse and type check Veltrano code
pub fn parse_and_type_check(
    code: &str,
    config: Config,
) -> Result<
    (
        Program,
        std::collections::HashMap<usize, veltrano::type_checker::MethodResolution>,
    ),
    VeltranoError,
> {
    let program = parse_veltrano_code(code, config)?;

    let mut type_checker = VeltranoTypeChecker::new();
    type_checker.check_program(&program).map_err(|errors| {
        // Convert the first TypeCheckError to VeltranoError
        errors
            .into_iter()
            .next()
            .map(Into::into)
            .unwrap_or_else(|| {
                VeltranoError::new(
                    veltrano::error::ErrorKind::TypeError,
                    "Unknown type checking error",
                )
            })
    })?;
    let resolutions = type_checker.get_method_resolutions().clone();

    Ok((program, resolutions))
}

/// Shared utility to perform full transpilation pipeline: lex → parse → type check → codegen
pub fn transpile(code: &str, ctx: &TestContext) -> Result<String, String> {
    let (program, resolutions) = if ctx.skip_type_check {
        (
            parse_veltrano_code(code, ctx.config.clone()).map_err(|e| e.to_string())?,
            std::collections::HashMap::new(),
        )
    } else {
        parse_and_type_check(code, ctx.config.clone())
            .map_err(|e| e.to_string())?
    };

    Ok(generate_rust_code(
        &program,
        ctx.config.clone(),
        Some(resolutions),
    ))
}

/// Format type checking errors into a user-friendly message
fn format_type_check_errors(errors: Vec<TypeCheckError>) -> String {
    let error_messages: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
    format!("Type checking failed: {}", error_messages.join(", "))
}

/// Build a detailed diff error message for transpilation mismatches
fn build_diff_error_message(context: &str, expected_rust: &str, actual_rust: &str) -> String {
    let mut error_msg = format!("\n=== EXPECTED OUTPUT MISMATCH ===\n{}\n", context);
    error_msg.push_str(&format!("\n--- Expected ---\n{}", expected_rust));
    error_msg.push_str(&format!("\n--- Actual ---\n{}", actual_rust));
    error_msg.push_str("\n--- Diff ---");

    // Simple line-by-line diff
    let expected_lines: Vec<&str> = expected_rust.lines().collect();
    let actual_lines: Vec<&str> = actual_rust.lines().collect();
    let max_lines = expected_lines.len().max(actual_lines.len());

    for i in 0..max_lines {
        let expected_line = expected_lines.get(i).unwrap_or(&"<EOF>");
        let actual_line = actual_lines.get(i).unwrap_or(&"<EOF>");

        if expected_line != actual_line {
            error_msg.push_str(&format!("\nLine {}:", i + 1));
            error_msg.push_str(&format!("\n  - {}", expected_line));
            error_msg.push_str(&format!("\n  + {}", actual_line));
        }
    }

    error_msg
}

/// Helper function to separate imports from code
fn separate_imports_and_code(rust_code: &str) -> (String, String) {
    let mut imports = Vec::new();
    let mut code_lines = Vec::new();
    let mut in_imports = true;

    for line in rust_code.lines() {
        if in_imports && (line.starts_with("use ") || line.trim().is_empty()) {
            imports.push(line);
        } else {
            in_imports = false;
            code_lines.push(line);
        }
    }

    (imports.join("\n"), code_lines.join("\n"))
}

/// Helper function to compile Rust code with bumpalo dependency (optimized)
fn compile_with_bumpalo(rust_code: &str, _name: &str) -> Result<(), String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a hash of the code for caching
    let mut hasher = DefaultHasher::new();
    rust_code.hash(&mut hasher);
    let code_hash = hasher.finish();

    // Use a simpler temp file approach with caching
    let temp_dir = format!("/tmp/veltrano_cache_{:x}", code_hash);
    let src_dir = format!("{}/src", temp_dir);

    // Check if this exact code has already been compiled successfully
    if std::path::Path::new(&format!("{}/.compiled_ok", temp_dir)).exists() {
        return Ok(());
    }

    // Create directory structure only if it doesn't exist
    if !std::path::Path::new(&temp_dir).exists() {
        fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

        // Create Cargo.toml with bumpalo dependency
        let cargo_toml = r#"[package]
name = "veltrano_test"
version = "0.1.0"
edition = "2021"

[dependencies]
bumpalo = "3.0"
"#;

        fs::write(format!("{}/Cargo.toml", temp_dir), cargo_toml)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

        // Create main.rs with the generated code
        fs::write(format!("{}/src/main.rs", temp_dir), rust_code)
            .map_err(|e| format!("Failed to write main.rs: {}", e))?;

        // Run cargo check to verify compilation
        let output = Command::new("cargo")
            .arg("check")
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| format!("Failed to execute cargo: {}", e))?;

        if !output.status.success() {
            // Clean up on failure
            let _ = fs::remove_dir_all(&temp_dir);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Compilation failed:\n{}", stderr));
        } else {
            // Mark as successfully compiled
            let _ = fs::write(format!("{}/.compiled_ok", temp_dir), "");
        }
    }

    Ok(())
}

/// Full transpilation and compilation pipeline with variable injections
pub fn transpile_and_compile(code: &str, ctx: &TestContext) -> Result<String, String> {
    let rust_code = transpile(code, ctx)?;

    // Wrap in main function if needed
    let complete_rust_code = if rust_code.contains("fn main") {
        rust_code.clone()
    } else {
        let (imports, code) = separate_imports_and_code(&rust_code);

        // Build variable injections
        let mut injections = String::from("    let bump = &bumpalo::Bump::new();\n");

        // Special case handling for common patterns
        if code.contains("if x") && !ctx.variable_injections.iter().any(|(name, _)| name == "x") {
            injections.push_str("    let x = 10;\n");
        }
        if code.contains("while counter")
            && !ctx
                .variable_injections
                .iter()
                .any(|(name, _)| name == "counter")
        {
            injections.push_str("    let counter = 0;\n");
        }

        // Add custom variable injections
        for (var_name, init_code) in &ctx.variable_injections {
            if code.contains(var_name) {
                injections.push_str(&format!("    {}\n", init_code));
            }
        }

        format!("{}\n\nfn main() {{\n{}{}\n}}", imports, injections, code)
    };

    compile_with_bumpalo(&complete_rust_code, &ctx.test_name)?;
    Ok(rust_code)
}

/// Wrap Rust code in a main function if needed
fn wrap_in_main_if_needed(rust_code: &str) -> String {
    if rust_code.contains("fn main") {
        rust_code.to_string()
    } else {
        let (imports, code) = separate_imports_and_code(rust_code);
        format!("{}\n\nfn main() {{\n{}\n}}", imports, code)
    }
}

/// Helper to compile already-generated Rust code with proper wrapping
pub fn compile_rust_code(rust_code: &str, ctx: &TestContext) -> Result<(), String> {
    // Optionally remove lines with intentional errors (marked with // ERROR comment)
    let cleaned_rust_code = if ctx.remove_error_lines {
        rust_code
            .lines()
            .filter(|line| !line.contains("// ERROR"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        rust_code.to_string()
    };

    let complete_rust_code = wrap_in_main_if_needed(&cleaned_rust_code);
    compile_with_bumpalo(&complete_rust_code, &ctx.test_name)
}

/// Helper to assert parsing fails with optional specific error message
pub fn assert_parse_error(code: &str, ctx: &TestContext) -> VeltranoError {
    match parse_veltrano_code(code, ctx.config.clone()) {
        Ok(_) => panic!("Expected parsing to fail, but it succeeded"),
        Err(error) => {
            let error_string = error.to_string();
            match &ctx.expected_error {
                Some(expected) if !error_string.contains(expected) => {
                    panic!(
                        "Expected error containing '{}', but got: '{}'",
                        expected, error_string
                    )
                }
                _ => error,
            }
        }
    }
}

/// Helper to assert transpilation output matches expected result
pub fn assert_transpilation_match(
    veltrano_code: &str,
    expected_rust: &str,
    ctx: &TestContext,
) {
    let actual_rust = match transpile(veltrano_code, ctx) {
        Ok(rust) => rust,
        Err(e) => panic!("Transpilation failed: {}", e),
    };

    // Compare with trimmed whitespace to handle trailing newlines
    if actual_rust.trim() != expected_rust.trim() {
        panic!(
            "\nVeltrano code:\n{}\n\nExpected Rust:\n{}\n\nActual Rust:\n{}",
            veltrano_code, expected_rust, actual_rust
        );
    }
}

/// Helper to assert transpilation output with detailed diff reporting
pub fn assert_transpilation_output(
    veltrano_code: &str,
    expected_rust: &str,
    ctx: &TestContext,
    context: &str, // For error reporting (e.g., "file: example.vl, config: tuf")
) {
    let actual_rust = match transpile(veltrano_code, ctx) {
        Ok(rust) => rust,
        Err(e) => panic!("Transpilation failed: {}", e),
    };

    // Compare output (trim to handle trailing newlines)
    if actual_rust.trim() != expected_rust.trim() {
        panic!("{}", build_diff_error_message(
            context,
            expected_rust,
            &actual_rust,
        ));
    }
}

/// Helper to assert type checking fails with optional specific error message
pub fn assert_type_check_error(code: &str, ctx: &TestContext) -> String {
    match parse_and_type_check(code, ctx.config.clone()) {
        Ok(_) => panic!("Expected type checking to fail, but it succeeded"),
        Err(error) => {
            let error_message = format!("{}: {}", error.kind, error.message);
            match &ctx.expected_error {
                Some(expected) if !error_message.contains(expected) => {
                    panic!(
                        "Expected error containing '{}', but got: '{}'",
                        expected, error_message
                    )
                }
                _ => error_message,
            }
        }
    }
}

/// Helper to assert either parsing or type checking fails with optional specific error message
/// This tries parsing first, and if that succeeds, tries type checking
pub fn assert_parse_or_type_check_error(code: &str, ctx: &TestContext) -> String {
    // First try parsing only
    match parse_veltrano_code(code, ctx.config.clone()) {
        Err(parse_error) => {
            // Parse failed - check if this matches expected error
            let error_string = parse_error.to_string();
            match &ctx.expected_error {
                Some(expected) if !error_string.contains(expected) => {
                    panic!(
                        "Expected error containing '{}', but got: '{}'",
                        expected, error_string
                    )
                }
                _ => error_string,
            }
        }
        Ok(_) => {
            // Parse succeeded - try type checking
            assert_type_check_error(code, ctx)
        }
    }
}
