use crate::codegen::CodeGenerator;
use crate::config::Config;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::fs;
use std::process::Command;

// Helper function to separate imports from code
fn separate_imports_and_code(rust_code: &str) -> (String, String) {
    let lines: Vec<&str> = rust_code.lines().collect();
    let mut imports = Vec::new();
    let mut code_lines = Vec::new();
    let mut in_imports = true;

    for line in lines {
        if in_imports && (line.starts_with("use ") || line.trim().is_empty()) {
            imports.push(line);
        } else {
            in_imports = false;
            code_lines.push(line);
        }
    }

    (imports.join("\n"), code_lines.join("\n"))
}

// Helper function to compile Rust code with bumpalo dependency (optimized)
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

#[test]
fn test_camel_to_snake_case() {
    let codegen = CodeGenerator::with_config(Config::default());

    assert_eq!(codegen.camel_to_snake_case("camelCase"), "camel_case");
    assert_eq!(codegen.camel_to_snake_case("CamelCase"), "_camel_case");
    assert_eq!(codegen.camel_to_snake_case("simpleVar"), "simple_var");
    assert_eq!(
        codegen.camel_to_snake_case("veryLongCamelCaseVariableName"),
        "very_long_camel_case_variable_name"
    );
    assert_eq!(codegen.camel_to_snake_case("a"), "a");
    assert_eq!(codegen.camel_to_snake_case("aB"), "a_b");
    assert_eq!(codegen.camel_to_snake_case("aBc"), "a_bc");
    assert_eq!(codegen.camel_to_snake_case("XMLParser"), "_x_m_l_parser");
    assert_eq!(
        codegen.camel_to_snake_case("httpURLConnection"),
        "http_u_r_l_connection"
    );
    assert_eq!(codegen.camel_to_snake_case("main"), "main");
    assert_eq!(codegen.camel_to_snake_case("calculateSum"), "calculate_sum");
    assert_eq!(codegen.camel_to_snake_case("calculate_sum"), "calculate__sum");
    assert_eq!(codegen.camel_to_snake_case("calculate_Sum"), "calculate___sum");
}

#[test]
fn test_camel_case_transpilation() {
    let source = r#"
fun calculateSum(firstNumber: Int, secondNumber: Int): Int {
    val totalResult: Int = firstNumber + secondNumber
    return totalResult
}
"#;

    let config = Config {
        preserve_comments: true,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(Config {
        preserve_comments: true,
    });
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
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let examples = extract_code_examples(&readme_content);

    // Check expected count of transpilation examples (Veltrano -> Rust pairs)
    assert_eq!(
        examples.len(), 3,
        "Expected 3 transpilation examples in README, found {}. If you added/removed examples, update this count.",
        examples.len()
    );

    for (veltrano_code, expected_rust) in examples {
        let config = Config {
            preserve_comments: true,
        };
        let mut lexer = Lexer::with_config(veltrano_code.clone(), config.clone());
        let all_tokens = lexer.tokenize();
        let mut parser = Parser::new(all_tokens);

        if let Ok(program) = parser.parse() {
            let mut codegen = CodeGenerator::with_config(Config {
                preserve_comments: true,
            });
            let actual_rust = codegen.generate(&program);

            // Compare with trimmed whitespace to handle trailing newlines
            assert_eq!(
                actual_rust.trim(),
                expected_rust.trim(),
                "\nVeltrano code:\n{}\n\nExpected Rust:\n{}\n\nActual Rust:\n{}",
                veltrano_code,
                expected_rust,
                actual_rust
            );
        }
    }
}

#[test]
fn test_readme_rust_outputs_compile() {
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");
    let rust_examples = extract_rust_code_examples(&readme_content);

    // Check expected count of Rust code blocks
    assert_eq!(
        rust_examples.len(), 4,
        "Expected 4 Rust code examples in README, found {}. If you added/removed examples, update this count.",
        rust_examples.len()
    );

    for (index, rust_code) in rust_examples.iter().enumerate() {
        // Remove lines with intentional errors (marked with // ERROR comment)
        let cleaned_rust_code = rust_code
            .lines()
            .filter(|line| !line.contains("// ERROR"))
            .collect::<Vec<_>>()
            .join("\n");

        // Wrap the code in a main function if it's not already a complete program
        let complete_rust_code = if cleaned_rust_code.contains("fn main") {
            cleaned_rust_code.clone()
        } else {
            // Separate imports from code
            let (imports, code) = separate_imports_and_code(&cleaned_rust_code);
            format!("{}\n\nfn main() {{\n{}\n}}", imports, code)
        };

        // Try to compile the Rust code with bumpalo support
        if let Err(error) = compile_with_bumpalo(&complete_rust_code, &format!("readme_{}", index))
        {
            panic!(
                "README Rust example {} failed to compile:\n{}\n\nCode:\n{}",
                index, error, complete_rust_code
            );
        }
    }
}

#[test]
fn test_readme_veltrano_snippets_transpile_and_compile() {
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");
    let veltrano_examples = extract_veltrano_code_examples(&readme_content);

    // Check expected count of standalone Veltrano code blocks
    assert_eq!(
        veltrano_examples.len(), 12,
        "Expected 12 Veltrano code examples in README, found {}. If you added/removed examples, update this count.",
        veltrano_examples.len()
    );

    for (index, veltrano_code) in veltrano_examples.iter().enumerate() {
        // Skip examples that are marked as Kotlin (not Veltrano)
        if veltrano_code.trim().starts_with("// Kotlin") {
            println!("Skipping Kotlin example {} (not Veltrano)", index);
            continue;
        }

        // Try to transpile the Veltrano code
        let config = Config {
            preserve_comments: true,
        };
        let mut lexer = Lexer::with_config(veltrano_code.clone(), config.clone());
        let all_tokens = lexer.tokenize();
        let mut parser = Parser::new(all_tokens);

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
        let mut codegen = CodeGenerator::with_config(Config {
            preserve_comments: true,
        });
        let rust_code = codegen.generate(&program);

        // Create a temporary Rust file
        let temp_file = format!("/tmp/readme_veltrano_example_{}.rs", index);

        // Wrap the code in a main function if it's not already a complete program
        let complete_rust_code = if rust_code.contains("fn main") {
            rust_code.clone()
        } else {
            // Separate imports from code
            let (imports, code) = separate_imports_and_code(&rust_code);

            // Special case for control flow examples that use undefined variables
            let main_body = if code.contains("if x") {
                format!(
                    "    let bump = &bumpalo::Bump::new();\n    let x = 10;\n{}",
                    code
                )
            } else if code.contains("while counter") {
                format!(
                    "    let bump = &bumpalo::Bump::new();\n    let counter = 0;\n{}",
                    code
                )
            } else {
                format!("    let bump = &bumpalo::Bump::new();\n{}", code)
            };

            format!("{}\n\nfn main() {{\n{}\n}}", imports, main_body)
        };

        fs::write(&temp_file, &complete_rust_code)
            .expect(&format!("Failed to write temp file {}", temp_file));

        // Try to compile the generated Rust code with bumpalo support
        if let Err(error) =
            compile_with_bumpalo(&complete_rust_code, &format!("readme_veltrano_{}", index))
        {
            panic!(
                "README Veltrano example {} transpiled but failed to compile:\n{}\n\nVeltrano code:\n{}\n\nGenerated Rust code:\n{}",
                index, error, veltrano_code, complete_rust_code
            );
        }

        // Clean up temporary files
        let _ = fs::remove_file(&temp_file);
    }
}

fn test_examples_with_config(preserve_comments: bool) {
    // Dynamically discover all .vl files in the examples directory
    let examples_dir = std::path::Path::new("examples");
    let example_files: Vec<_> = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "vl" {
                let filename = path.file_name()?.to_string_lossy().into_owned();
                // Skip test files that are meant to fail
                if filename.contains(".fail.") {
                    None
                } else {
                    Some(filename)
                }
            } else {
                None
            }
        })
        .collect();

    for example_file in &example_files {
        let example_path = format!("examples/{}", example_file);
        let veltrano_code =
            fs::read_to_string(&example_path).expect(&format!("Failed to read {}", example_path));

        let config = Config { preserve_comments };
        let mut lexer = Lexer::with_config(veltrano_code.clone(), config.clone());
        let all_tokens = lexer.tokenize();
        let mut parser = Parser::new(all_tokens);

        let program = match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                panic!("Example {} failed to parse: {}", example_file, err);
            }
        };

        // Generate Rust code
        let mut codegen = CodeGenerator::with_config(config);
        let rust_code = codegen.generate(&program);

        // Create a temporary Rust file
        let comments_suffix = if preserve_comments {
            "_with_comments"
        } else {
            "_no_comments"
        };
        let temp_file = format!(
            "/tmp/example_{}{}.rs",
            example_file.replace(".vl", ""),
            comments_suffix
        );

        // Wrap the code in a main function if it's not already a complete program
        let complete_rust_code = if rust_code.contains("fn main") {
            rust_code.clone()
        } else {
            // Separate imports from code
            let (imports, code) = separate_imports_and_code(&rust_code);
            format!(
                "{}\n\nfn main() {{\n    let bump = &bumpalo::Bump::new();\n{}\n}}",
                imports, code
            )
        };

        fs::write(&temp_file, &complete_rust_code)
            .expect(&format!("Failed to write temp file {}", temp_file));

        // Try to compile the generated Rust code with bumpalo support
        let test_name = format!(
            "example_{}{}",
            example_file.replace(".vl", ""),
            comments_suffix
        );
        if let Err(error) = compile_with_bumpalo(&complete_rust_code, &test_name) {
            panic!(
                "Example {} (preserve_comments={}) transpiled but failed to compile:\n{}\n\nVeltrano code:\n{}\n\nGenerated Rust code:\n{}",
                example_file, preserve_comments, error, veltrano_code, complete_rust_code
            );
        }

        // Clean up temporary files
        let _ = fs::remove_file(&temp_file);
    }
}

#[test]
fn test_examples_transpile_and_compile_preserve_comments_false() {
    test_examples_with_config(false);
}

#[test]
fn test_examples_transpile_and_compile_preserve_comments_true() {
    test_examples_with_config(true);
}

fn extract_code_examples(readme: &str) -> Vec<(String, String)> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = readme.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for "Transpiles to:" or "Generated Output" followed by rust code
        if lines[i].contains("**Transpiles to:**")
            || lines[i].contains("transpiles to:**")
            || lines[i].contains("**Generated Output")
        {
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
                if lines[j].contains("**Transpiles to:**")
                    || lines[j].contains("**Examples:**")
                    || lines[j].contains("**Example Input")
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
                    if lines[j].contains("**Transpiles to:**") || lines[j].contains("**Output") {
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

fn extract_table_examples(readme: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = readme.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for table rows with Veltrano code examples
        if line.contains("|") && line.contains("val ") {
            // Extract the example column (typically the last column)
            let columns: Vec<&str> = line.split('|').collect();
            if columns.len() >= 4 {
                let example_column = columns[columns.len() - 2].trim(); // Second to last column (last is usually empty)

                // Look for Veltrano code patterns
                if example_column.starts_with("`val ") && example_column.ends_with("`") {
                    // Extract the code between backticks
                    let code = example_column.trim_start_matches('`').trim_end_matches('`');
                    if !code.is_empty() {
                        examples.push(code.to_string());
                    }
                }
            }
        }
        i += 1;
    }

    examples
}

#[test]
fn test_while_true_to_loop_conversion() {
    // Test that while(true) converts to loop
    let veltrano_code = r#"fun infiniteLoop(): Nothing {
    while (true) {
        val x = 42
    }
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);

    let program = parser.parse().expect("Failed to parse");
    let mut codegen = CodeGenerator::with_config(config.clone());
    let actual_rust = codegen.generate(&program);

    // Check that the output contains "loop" instead of "while true"
    assert!(
        actual_rust.contains("loop {"),
        "Expected 'loop {{' but got: {}",
        actual_rust
    );
    assert!(
        !actual_rust.contains("while true"),
        "Should not contain 'while true', got: {}",
        actual_rust
    );
}

#[test]
fn test_inline_comments_with_and_without_preservation() {
    let veltrano_code = r#"fun main() {
    val simple: Int = 42 // Simple inline comment
    // var mutable: Bool = true // Another inline comment
    val string: Str = "hello" // String with inline comment
    
    // Full line comment
    val complex: Own<String> = "test".toString() // Method call with comment
    
    if (simple > 0) { // Inline comment after condition
        println("{}", simple) // Comment in block
    } else {
        println("negative") // Comment in else block
    }
    
    while (true) { // Loop with inline comment
        println("infinite loop") // Comment in loop body
        break // Break to avoid infinite loop
    }
}"#;

    // Test with comment preservation enabled
    let config_with_comments = Config {
        preserve_comments: true,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config_with_comments.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);
    let program = parser
        .parse()
        .expect("Failed to parse inline comments test");
    let mut codegen = CodeGenerator::with_config(config_with_comments);
    let rust_code_with_comments = codegen.generate(&program);

    // Check that all comments are preserved
    assert!(rust_code_with_comments.contains("// Simple inline comment"));
    assert!(rust_code_with_comments.contains("// Another inline comment"));
    assert!(rust_code_with_comments.contains("// String with inline comment"));
    assert!(rust_code_with_comments.contains("// Full line comment"));
    assert!(rust_code_with_comments.contains("// Method call with comment"));
    assert!(rust_code_with_comments.contains("// Inline comment after condition"));
    assert!(rust_code_with_comments.contains("// Comment in block"));
    assert!(rust_code_with_comments.contains("// Comment in else block"));
    assert!(rust_code_with_comments.contains("// Loop with inline comment"));
    assert!(rust_code_with_comments.contains("// Comment in loop body"));
    assert!(rust_code_with_comments.contains("// Break to avoid infinite loop"));

    // Test with comment preservation disabled
    let config_no_comments = Config {
        preserve_comments: false,
    };
    let mut lexer2 = Lexer::with_config(veltrano_code.to_string(), config_no_comments.clone());
    let all_tokens2 = lexer2.tokenize();
    let mut parser2 = Parser::new(all_tokens2);
    let program2 = parser2
        .parse()
        .expect("Failed to parse inline comments test");
    let mut codegen2 = CodeGenerator::with_config(config_no_comments);
    let rust_code_no_comments = codegen2.generate(&program2);

    // Check that NO comments are preserved
    assert!(!rust_code_no_comments.contains("// Simple inline comment"));
    assert!(!rust_code_no_comments.contains("// Another inline comment"));
    assert!(!rust_code_no_comments.contains("// String with inline comment"));
    assert!(!rust_code_no_comments.contains("// Full line comment"));
    assert!(!rust_code_no_comments.contains("// Method call with comment"));
    assert!(!rust_code_no_comments.contains("// Inline comment after condition"));
    assert!(!rust_code_no_comments.contains("// Comment in block"));
    assert!(!rust_code_no_comments.contains("// Comment in else block"));
    assert!(!rust_code_no_comments.contains("// Loop with inline comment"));
    assert!(!rust_code_no_comments.contains("// Comment in loop body"));
    assert!(!rust_code_no_comments.contains("// Break to avoid infinite loop"));

    // Verify the code structure is the same (minus comments)
    assert!(rust_code_with_comments.contains("let simple: i64 = 42"));
    assert!(rust_code_no_comments.contains("let simple: i64 = 42"));
    // assert!(rust_code_with_comments.contains("let mut mutable: bool = true"));
    // assert!(rust_code_no_comments.contains("let mut mutable: bool = true"));
}

#[test]
fn test_mut_ref_type_and_method() {
    // Test MutRef type annotation
    let veltrano_code = r#"fun testMutRef() {
    val value: MutRef<Int> = someVar.mutRef()
    val strRef: MutRef<Own<String>> = text.mutRef()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);

    let program = parser.parse().expect("Failed to parse MutRef test");
    let mut codegen = CodeGenerator::with_config(config.clone());
    let rust_code = codegen.generate(&program);

    // Check that MutRef<T> becomes &mut T (no automatic .clone())
    assert!(
        rust_code.contains("let value: &mut i64 = &mut some_var"),
        "Expected 'let value: &mut i64 = &mut some_var' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let str_ref: &mut String = &mut text"),
        "Expected 'let str_ref: &mut String = &mut text' but got: {}",
        rust_code
    );

    // Test .mutRef() method call without type annotation
    let veltrano_code2 = r#"fun testMutRefMethod() {
    val mutableRef = number.mutRef()
    val another = "test".mutRef()
}"#;

    let mut lexer2 = Lexer::with_config(veltrano_code2.to_string(), config.clone());
    let all_tokens2 = lexer2.tokenize();
    let mut parser2 = Parser::new(all_tokens2);
    let program2 = parser2.parse().expect("Failed to parse mutRef method test");
    let mut codegen2 = CodeGenerator::with_config(config.clone());
    let rust_code2 = codegen2.generate(&program2);

    // Check that .mutRef() becomes &mut x (no automatic .clone())
    assert!(
        rust_code2.contains("let mutable_ref = &mut number"),
        "Expected 'let mutable_ref = &mut number' but got: {}",
        rust_code2
    );
    assert!(
        rust_code2.contains("let another = &mut \"test\""),
        "Expected 'let another = &mut \"test\"' but got: {}",
        rust_code2
    );
}

#[test]
fn test_own_value_type_validation() {
    // Test that Own<Int> is rejected
    let veltrano_code = r#"fun main() {
    val x: Own<Int> = 42
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);

    let result = parser.parse();
    assert!(result.is_err(), "Expected parse error for Own<Int>");
    assert!(
        result
            .unwrap_err()
            .contains("Cannot use Own<Int>. Int is already owned"),
        "Expected error message about Own<Int>"
    );

    // Test that Own<Bool> is rejected
    let veltrano_code2 = r#"fun main() {
    val flag: Own<Bool> = true
}"#;

    let mut lexer2 = Lexer::with_config(veltrano_code2.to_string(), config.clone());
    let all_tokens2 = lexer2.tokenize();
    let mut parser2 = Parser::new(all_tokens2);

    let result2 = parser2.parse();
    assert!(result2.is_err(), "Expected parse error for Own<Bool>");

    // Test that Own<String> is accepted
    let veltrano_code3 = r#"fun main() {
    val text: Own<String> = "hello".toString()
}"#;

    let mut lexer3 = Lexer::with_config(veltrano_code3.to_string(), config.clone());
    let all_tokens3 = lexer3.tokenize();
    let mut parser3 = Parser::new(all_tokens3);

    let result3 = parser3.parse();
    assert!(result3.is_ok(), "Expected Own<String> to be accepted");

    // Test that Own<MutRef<T>> is rejected
    let veltrano_code4 = r#"fun main() {
    val x: Own<MutRef<String>> = something
}"#;

    let mut lexer4 = Lexer::with_config(veltrano_code4.to_string(), config.clone());
    let all_tokens4 = lexer4.tokenize();
    let mut parser4 = Parser::new(all_tokens4);

    let result4 = parser4.parse();
    assert!(result4.is_err(), "Expected parse error for Own<MutRef<T>>");
    assert!(
        result4.unwrap_err().contains("MutRef<T> is already owned"),
        "Expected error message about mutable references"
    );

    // Test that Own<Box<T>> is rejected
    let veltrano_code5 = r#"fun main() {
    val x: Own<Box<String>> = something
}"#;

    let mut lexer5 = Lexer::with_config(veltrano_code5.to_string(), config.clone());
    let all_tokens5 = lexer5.tokenize();
    let mut parser5 = Parser::new(all_tokens5);

    let result5 = parser5.parse();
    assert!(result5.is_err(), "Expected parse error for Own<Box<T>>");
    assert!(
        result5.unwrap_err().contains("Box<T> is already owned"),
        "Expected error message about Box already being owned"
    );

    // Test that Own<Own<T>> is rejected
    let veltrano_code6 = r#"fun main() {
    val x: Own<Own<String>> = something
}"#;

    let mut lexer6 = Lexer::with_config(veltrano_code6.to_string(), config.clone());
    let all_tokens6 = lexer6.tokenize();
    let mut parser6 = Parser::new(all_tokens6);

    let result6 = parser6.parse();
    assert!(result6.is_err(), "Expected parse error for Own<Own<T>>");
    assert!(
        result6
            .unwrap_err()
            .contains("Cannot use Own<> on already owned type"),
        "Expected error message about Own on already owned type"
    );
}

#[test]
fn test_fail_examples() {
    // Test that .fail.vl files actually fail to parse with expected errors
    let examples_dir = std::path::Path::new("examples");
    let fail_files: Vec<_> = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "vl" {
                let filename = path.file_name()?.to_string_lossy().into_owned();
                // Only include test files that are meant to fail
                if filename.contains(".fail.") {
                    Some(filename)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    for fail_file in &fail_files {
        let example_path = format!("examples/{}", fail_file);
        let veltrano_code =
            fs::read_to_string(&example_path).expect(&format!("Failed to read {}", example_path));

        // Extract expected error from first line comment if present
        let expected_error = veltrano_code.lines().next().and_then(|line| {
            if line.starts_with("// Expected error:") {
                Some(line.trim_start_matches("// Expected error:").trim())
            } else {
                None
            }
        });

        let config = Config {
            preserve_comments: false,
        };
        let mut lexer = Lexer::with_config(veltrano_code.clone(), config.clone());
        let all_tokens = lexer.tokenize();
        let mut parser = Parser::new(all_tokens);

        let result = parser.parse();
        assert!(
            result.is_err(),
            "Expected {} to fail parsing, but it succeeded",
            fail_file
        );

        // Check for expected error message if specified
        if let Some(expected) = expected_error {
            let actual_error = result.unwrap_err();
            assert!(
                actual_error.contains(expected),
                "Expected error containing '{}' for {}, but got: '{}'",
                expected,
                fail_file,
                actual_error
            );
        }
    }
}

#[test]
fn test_clone_ufcs_generation() {
    // Test that clone() generates UFCS syntax
    let veltrano_code = r#"fun testClone() {
    val owned: Own<String> = "hello".toString()
    val borrowed: String = owned.ref()
    
    // Test various clone scenarios
    val clonedOwned = owned.clone()
    val clonedBorrowed = borrowed.clone()
    
    // Test with method chaining
    val chained = owned.ref().clone()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);

    let program = parser.parse().expect("Failed to parse clone test");
    let mut codegen = CodeGenerator::with_config(config.clone());
    let rust_code = codegen.generate(&program);

    // Check UFCS generation for clone
    assert!(
        rust_code.contains("let cloned_owned = Clone::clone(owned)"),
        "Expected 'Clone::clone(owned)' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let cloned_borrowed = Clone::clone(borrowed)"),
        "Expected 'Clone::clone(borrowed)' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let chained = Clone::clone(&owned)"),
        "Expected 'Clone::clone(&owned)' for chained call but got: {}",
        rust_code
    );
}

#[test]
fn test_mut_ref_function() {
    // Test that MutRef() generates &mut (&value).clone()
    let veltrano_code = r#"fun testMutRef() {
    val owned: Own<String> = "hello".toString()
    val borrowed: String = owned.ref()
    
    // Test MutRef() function
    val mutRefOwned = MutRef(owned)
    val mutRefBorrowed = MutRef(borrowed)
    
    // Test with literals
    val mutRefLiteral = MutRef(42)
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);

    let program = parser
        .parse()
        .expect("Failed to parse MutRef function test");
    let mut codegen = CodeGenerator::with_config(config.clone());
    let rust_code = codegen.generate(&program);

    // Check &mut (&value).clone() generation
    assert!(
        rust_code.contains("let mut_ref_owned = &mut (&owned).clone()"),
        "Expected '&mut (&owned).clone()' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let mut_ref_borrowed = &mut (&borrowed).clone()"),
        "Expected '&mut (&borrowed).clone()' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let mut_ref_literal = &mut (&42).clone()"),
        "Expected '&mut (&42).clone()' but got: {}",
        rust_code
    );
}

#[test]
fn test_mutref_method_chaining() {
    // Test that .mutRef() method works well with chaining
    let veltrano_code = r#"fun testChaining() {
    val owned: Own<String> = "hello".toString()
    val borrowed: String = owned.ref()
    
    // Test method chaining patterns
    val chained1 = owned.clone().mutRef()
    val chained2 = borrowed.clone().ref().mutRef()
    
    // Direct .mutRef() usage
    val directMut = owned.mutRef()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);

    let program = parser
        .parse()
        .expect("Failed to parse method chaining test");
    let mut codegen = CodeGenerator::with_config(config.clone());
    let rust_code = codegen.generate(&program);

    // Check chaining patterns
    assert!(
        rust_code.contains("let chained1 = &mut Clone::clone(owned)"),
        "Expected '&mut Clone::clone(owned)' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let chained2 = &mut &Clone::clone(borrowed)"),
        "Expected '&mut &Clone::clone(borrowed)' but got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let direct_mut = &mut owned"),
        "Expected '&mut owned' but got: {}",
        rust_code
    );
}

#[test]
fn test_readme_table_examples() {
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");
    let table_examples = extract_table_examples(&readme_content);

    // Check expected count of table examples
    assert_eq!(
        table_examples.len(), 4,
        "Expected 4 table examples in README, found {}. If you added/removed examples, update this count.",
        table_examples.len()
    );

    for (index, example) in table_examples.iter().enumerate() {
        let config = Config {
            preserve_comments: false,
        };
        let mut lexer = Lexer::with_config(example.clone(), config.clone());
        let all_tokens = lexer.tokenize();
        let mut parser = Parser::new(all_tokens);

        let program = match parser.parse() {
            Ok(program) => program,
            Err(_err) => {
                // Some table examples might be fragments, let's try wrapping in a function
                let wrapped_example = format!("fun main() {{\n    {}\n}}", example);
                let mut lexer2 = Lexer::with_config(wrapped_example.clone(), config.clone());
                let all_tokens2 = lexer2.tokenize();
                let mut parser2 = Parser::new(all_tokens2);

                match parser2.parse() {
                    Ok(program) => program,
                    Err(_) => {
                        println!("Skipping table example {} (fragment): {}", index, example);
                        continue;
                    }
                }
            }
        };

        // Generate Rust code
        let mut codegen = CodeGenerator::with_config(config.clone());
        let rust_code = codegen.generate(&program);

        // Create a temporary Rust file
        let temp_file = format!("/tmp/table_example_{}.rs", index);

        // Wrap the code in a main function if it's not already a complete program
        let complete_rust_code = if rust_code.contains("fn main") {
            rust_code.clone()
        } else {
            // Separate imports from code
            let (imports, code) = separate_imports_and_code(&rust_code);

            // Add common variable declarations for table examples
            let mut vars = String::new();
            if code.contains("owned") {
                vars.push_str("    let owned = String::from(\"example\");\n");
            }
            if code.contains("borrowed") {
                vars.push_str("    let borrowed = &String::from(\"example\");\n");
            }
            if code.contains("num") {
                vars.push_str("    let num = 42i64;\n");
            }
            if code.contains("&s") && !code.contains("let s") {
                vars.push_str("    let s = &String::from(\"example\");\n");
            }

            format!(
                "{}\n\nfn main() {{\n{}    let bump = &bumpalo::Bump::new();\n{}}}",
                imports, vars, code
            )
        };

        fs::write(&temp_file, &complete_rust_code)
            .expect(&format!("Failed to write temp file {}", temp_file));

        // Try to compile the generated Rust code with bumpalo support
        if let Err(error) = compile_with_bumpalo(&complete_rust_code, &format!("table_{}", index)) {
            panic!(
                "Table example {} failed to compile:\n{}\n\nVeltrano code:\n{}\n\nGenerated Rust code:\n{}",
                index, error, example, complete_rust_code
            );
        }

        // Clean up temporary files
        let _ = fs::remove_file(&temp_file);
    }
}

#[test]
fn test_unit_literal() {
    let source = r#"
fun main() {
    val x: Unit = Unit
    val y = Unit
    println("{:?}", x)
    println("{:?}", y)
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check that Unit literal is transpiled to ()
    assert!(rust_code.contains("let x: () = ()"));
    assert!(rust_code.contains("let y = ()"));
}

#[test]
fn test_unary_expressions() {
    let source = r#"
fun main() {
    val negative = -42
    val expr = -(2 + 3)
    val spaced = - 15
    val var_neg = -negative
    val parens = -(-20)  // OK with parentheses
    
    println("{}", negative)
    println("{}", expr)
    println("{}", spaced)
    println("{}", var_neg)
    println("{}", parens)
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check that unary expressions are correctly transpiled
    assert!(rust_code.contains("let negative = -42"));
    assert!(rust_code.contains("let expr = -(2 + 3)"));
    assert!(rust_code.contains("let spaced = -15")); // Space allowed
    assert!(rust_code.contains("let var__neg = -negative"));
    assert!(rust_code.contains("let parens = -(-20)")); // OK with parentheses
}

#[test]
fn test_double_minus_forbidden() {
    let source = r#"
fun main() {
    val bad = --5
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);

    let result = parser.parse();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Double minus (--) is not allowed"));
}

#[test]
fn test_import_statement() {
    let source = r#"
import Vec.new as newVec
import Vec.push
import String.len

fun main() {
    val items = newVec()
    items.push(42)
    val text: String = "hello"
    val length = text.len()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check that imports don't generate any Rust code
    assert!(!rust_code.contains("import"));

    // Check that method calls use UFCS
    assert!(rust_code.contains("Vec::new()")); // newVec() -> Vec::new()
    assert!(rust_code.contains("Vec::push(items, 42)"));
    assert!(rust_code.contains("String::len(text)"));
}

#[test]
fn test_preimported_methods() {
    let source = r#"
fun main() {
    val text: Own<String> = "Hello"
    val cloned = text.clone()
    val string = text.toString()
    val reference = text.ref()
    val mutable = text.mutRef()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check pre-imported methods
    assert!(rust_code.contains("Clone::clone(text)"));
    assert!(rust_code.contains("ToString::to_string(text)"));
    assert!(rust_code.contains("&text")); // .ref() is now just borrowing
    assert!(rust_code.contains("&mut text")); // .mutRef()
}

#[test]
fn test_import_priority_over_preimported() {
    let source = r#"
import MyClone.clone

fun main() {
    val value = 42
    val cloned = value.clone()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Debug: print the generated code
    if rust_code.contains("Clone::clone") {
        println!("Found Clone::clone in generated code:\n{}", rust_code);
    }

    // Check that explicit import overrides pre-imported clone
    assert!(rust_code.contains("MyClone::clone(value)"));
}

#[test]
fn test_import_with_alias() {
    let source = r#"
import ToString.toString as stringify

fun main() {
    val num = 42
    val str = num.stringify()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check that alias works and maps to correct UFCS call
    assert!(rust_code.contains("ToString::to_string(num)"));
}

#[test]
fn test_local_function_priority_over_import() {
    let source = r#"
import Vec.new

fun main() {
    val result = new()
}

fun new(): Int {
    return 42
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Debug: print the generated code
    println!("Generated Rust code:\n{}", rust_code);

    // Check that local function is called, not Vec::new
    assert!(rust_code.contains("let result = new("));
    assert!(!rust_code.contains("Vec::new"));
}

#[test]
fn test_data_class_generation() {
    // Test data class with value types only (no lifetime needed)
    let source1 = r#"
data class Point(val x: Int, val y: Int)
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source1.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config.clone());
    let rust_code = codegen.generate(&program);

    // Check struct generation without lifetime
    assert!(rust_code.contains("#[derive(Debug, Clone)]"));
    assert!(rust_code.contains("pub struct Point {"));
    assert!(rust_code.contains("pub x: i64,"));
    assert!(rust_code.contains("pub y: i64,"));
    assert!(
        !rust_code.contains("<'a>"),
        "Point should not have lifetime parameter"
    );

    // Test data class with reference types (lifetime needed)
    let source2 = r#"
data class Person(val name: String, val age: Int)
"#;

    let mut lexer2 = Lexer::with_config(source2.to_string(), config.clone());
    let tokens2 = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let program2 = parser2.parse().expect("Parse should succeed");

    let mut codegen2 = CodeGenerator::with_config(config.clone());
    let rust_code2 = codegen2.generate(&program2);

    // Check struct generation with lifetime
    assert!(rust_code2.contains("#[derive(Debug, Clone)]"));
    assert!(rust_code2.contains("pub struct Person<'a> {"));
    assert!(rust_code2.contains("pub name: &'a String,"));
    assert!(rust_code2.contains("pub age: i64,"));

    // Test data class with custom types
    let source3 = r#"
data class Container(val item: MyType, val count: Int)
"#;

    let mut lexer3 = Lexer::with_config(source3.to_string(), config.clone());
    let tokens3 = lexer3.tokenize();
    let mut parser3 = Parser::new(tokens3);
    let program3 = parser3.parse().expect("Parse should succeed");

    let mut codegen3 = CodeGenerator::with_config(config);
    let rust_code3 = codegen3.generate(&program3);

    // Check struct generation with custom type (needs lifetime)
    assert!(rust_code3.contains("pub struct Container<'a> {"));
    assert!(rust_code3.contains("pub item: &'a MyType,"));
}

#[test]
fn test_data_class_initialization() {
    // Test Kotlin-style struct initialization
    let source = r#"
data class Point(val x: Int, val y: Int)
data class Person(val name: String, val age: Int)

fun main() {
    val p1 = Point(x = 10, y = 20)
    val p2 = Person(name = "Alice", age = 30)
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check struct initialization syntax
    assert!(rust_code.contains("let p1 = Point { x: 10, y: 20 };"));
    assert!(rust_code.contains("let p2 = Person { name: \"Alice\", age: 30 };"));
}

#[test]
fn test_data_class_field_shorthand() {
    // Test Rust field shorthand syntax in struct initialization
    let source = r#"
data class Point(val x: Int, val y: Int)
data class Person(val name: Str, val age: Int)

fun main() {
    // All positional - uses field shorthand
    val x = 10
    val y = 20
    val p1 = Point(x, y)
    
    // Mixed positional and named
    val name = "Alice"
    val p2 = Person(name, age = 30)
    
    // All named
    val p3 = Person(name = "Bob", age = 25)
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check field shorthand syntax
    assert!(rust_code.contains("let p1 = Point { x, y };"));
    assert!(rust_code.contains("let p2 = Person { name, age: 30 };"));
    assert!(rust_code.contains("let p3 = Person { name: \"Bob\", age: 25 };"));
}

#[test]
fn test_data_class_argument_order() {
    // Test that named arguments can be provided in any order
    let source = r#"
data class Person(val name: Str, val age: Int)
data class Book(val title: Str, val author: Str, val pages: Int)

fun main() {
    // Arguments in declaration order
    val p1 = Person(name = "Alice", age = 30)
    
    // Arguments in reversed order
    val p2 = Person(age = 25, name = "Bob")
    
    // Mixed order for 3+ fields
    val b1 = Book(title = "Title", author = "Author", pages = 100)
    val b2 = Book(pages = 200, title = "Another", author = "Someone")
    val b3 = Book(author = "Writer", pages = 300, title = "Book")
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // All should generate correct struct initialization regardless of order
    assert!(rust_code.contains("let p1 = Person { name: \"Alice\", age: 30 };"));
    assert!(rust_code.contains("let p2 = Person { age: 25, name: \"Bob\" };"));
    assert!(
        rust_code.contains("let b1 = Book { title: \"Title\", author: \"Author\", pages: 100 };")
    );
    assert!(rust_code
        .contains("let b2 = Book { pages: 200, title: \"Another\", author: \"Someone\" };"));
    assert!(
        rust_code.contains("let b3 = Book { author: \"Writer\", pages: 300, title: \"Book\" };")
    );
}

#[test]
fn test_data_class_mixed_bare_named_args() {
    // Test that bare and named arguments can be mixed in any order
    let source = r#"
data class Person(val name: Str, val age: Int)
data class Book(val title: Str, val author: Str, val pages: Int)

fun main() {
    val name = "Alice"
    val age = 30
    val title = "Rust Book"
    val author = "Steve"
    val pages = 500
    
    // Bare first, named second
    val p1 = Person(name, age = 25)
    
    // Named first, bare second
    val p2 = Person(age = 35, name)
    
    // Multiple combinations with 3 fields
    val b1 = Book(title, author = "Carol", pages = 300)     // bare, named, named
    val b2 = Book(title = "Guide", author, pages = 400)     // named, bare, named  
    val b3 = Book(title = "Manual", author = "Bob", pages)  // named, named, bare
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Verify correct struct initialization with mixed bare/named args
    assert!(rust_code.contains("let p1 = Person { name, age: 25 };"));
    assert!(rust_code.contains("let p2 = Person { age: 35, name };"));
    assert!(rust_code.contains("let b1 = Book { title, author: \"Carol\", pages: 300 };"));
    assert!(rust_code.contains("let b2 = Book { title: \"Guide\", author, pages: 400 };"));
    assert!(rust_code.contains("let b3 = Book { title: \"Manual\", author: \"Bob\", pages };"));
}

#[test]
fn test_contextual_comment_indentation() {
    // Test that comments are properly indented based on their contextual nesting level
    let veltrano_code = r#"fun test() {
    // First level comment (4 spaces in source)
    if (true) {
        // Second level comment (8 spaces in source)
            // Extra indented comment (12 spaces = 8 base + 4 extra)
        val x = 42
    }
    // Back to first level (4 spaces in source)
}

fun main() {
// Top level comment (0 spaces in source)
    val y = 10
}"#;

    let config = Config {
        preserve_comments: true,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);
    let program = parser
        .parse()
        .expect("Failed to parse contextual indentation test");
    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check that comments have proper indentation without double indentation
    assert!(rust_code.contains("fn test() {\n    // First level comment"));
    assert!(rust_code.contains("    if true {\n        // Second level comment"));
    assert!(rust_code.contains("    }\n    // Back to first level"));
    assert!(rust_code.contains("    // Top level comment (0 spaces in source)"));

    // Verify that the extra indentation is preserved while base indentation is stripped
    let lines: Vec<&str> = rust_code.lines().collect();
    let first_level_comment = lines
        .iter()
        .find(|line| line.contains("First level comment"))
        .unwrap();
    let second_level_comment = lines
        .iter()
        .find(|line| line.contains("Second level comment"))
        .unwrap();
    let extra_indented_comment = lines
        .iter()
        .find(|line| line.contains("Extra indented comment"))
        .unwrap();
    let back_to_first_comment = lines
        .iter()
        .find(|line| line.contains("Back to first level"))
        .unwrap();
    let top_level_comment = lines
        .iter()
        .find(|line| line.contains("Top level comment"))
        .unwrap();

    // Count leading spaces to verify proper indentation
    assert_eq!(
        first_level_comment.len() - first_level_comment.trim_start().len(),
        4
    );
    assert_eq!(
        second_level_comment.len() - second_level_comment.trim_start().len(),
        8
    );
    assert_eq!(
        extra_indented_comment.len() - extra_indented_comment.trim_start().len(),
        12
    ); // 8 base + 4 extra
    assert_eq!(
        back_to_first_comment.len() - back_to_first_comment.trim_start().len(),
        4
    );
    assert_eq!(
        top_level_comment.len() - top_level_comment.trim_start().len(),
        4
    ); // Inside main() function
}

#[test]
fn test_data_class_field_access() {
    // Test field access for data classes
    let source = r#"
data class Point(val x: Int, val y: Int)
data class Person(val name: Str, val age: Int)

fun main() {
    val p = Point(x = 10, y = 20)
    val person = Person(name = "Alice", age = 30)
    
    // Field access
    val x = p.x
    val y = p.y
    val name = person.name
    val age = person.age
    
    // Chained field access
    val someX = p.x
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check field access generation
    assert!(rust_code.contains("let x = p.x;"));
    assert!(rust_code.contains("let y = p.y;"));
    assert!(rust_code.contains("let name = person.name;"));
    assert!(rust_code.contains("let age = person.age;"));
    assert!(rust_code.contains("let some_x = p.x;"));
}

#[test]
fn test_multiline_method_chains() {
    // Test that method chains can span multiple lines with dots on new lines
    let source = r#"
fun main() {
    val hello: Str = "Hello".bumpRef()
    
    // Single line chain
    val single: Str = hello.ref().bumpRef()
    
    // Multiline chain with dots on new lines
    val multi: Str = hello
        .ref()
        .bumpRef()
    
    // Mixed style
    val mixed: Str = hello.ref()
        .bumpRef()
    
    // Longer chains
    val long: Str = hello
        .ref()
        .ref()
        .ref()
        .bumpRef()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config.clone());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parse should succeed");

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // All variations should generate bump allocations
    assert!(rust_code.contains("let single: &str = bump.alloc(&hello);"));
    assert!(rust_code.contains("let multi: &str = bump.alloc(&hello);"));
    assert!(rust_code.contains("let mixed: &str = bump.alloc(&hello);"));
    assert!(rust_code.contains("let long: &str = bump.alloc(&&&hello);"));
}

#[test]
fn test_nested_function_call_comment_indentation() {
    // Test that comments in nested function calls have proper indentation
    let veltrano_code = r#"fun f(a: Int, b: Int): Int {
    return a + b
}

fun g(x: Int, y: Int, z: Int): Int {
    return x * y * z
}

fun main() {
    val result = f(
        g(
            1,
            // Nested level comment (base)
            2,
                // Nested level with extra indent
            3
        ),
        // Outer level comment
        4
    )
    
    // Even deeper nesting
    val deep = f(
        g(
            f(
                10,
                // Three levels deep
                20
            ),
            // Two levels deep
            30,
            40
        ),
        50
    )
}"#;

    let config = Config {
        preserve_comments: true,
    };
    let mut lexer = Lexer::with_config(veltrano_code.to_string(), config.clone());
    let all_tokens = lexer.tokenize();
    let mut parser = Parser::new(all_tokens);
    let program = parser
        .parse()
        .expect("Failed to parse nested function calls");
    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    // Check proper indentation at different nesting levels
    let lines: Vec<&str> = rust_code.lines().collect();

    // Find and verify comment indentations
    let nested_base = lines
        .iter()
        .find(|line| line.contains("Nested level comment (base)"))
        .expect("Should find nested base comment");
    let nested_extra = lines
        .iter()
        .find(|line| line.contains("Nested level with extra indent"))
        .expect("Should find nested extra comment");
    let outer = lines
        .iter()
        .find(|line| line.contains("Outer level comment"))
        .expect("Should find outer comment");
    let three_deep = lines
        .iter()
        .find(|line| line.contains("Three levels deep"))
        .expect("Should find three levels deep comment");
    let two_deep = lines
        .iter()
        .find(|line| line.contains("Two levels deep"))
        .expect("Should find two levels deep comment");

    // Count leading spaces
    let nested_base_indent = nested_base.len() - nested_base.trim_start().len();
    let nested_extra_indent = nested_extra.len() - nested_extra.trim_start().len();
    let outer_indent = outer.len() - outer.trim_start().len();
    let three_deep_indent = three_deep.len() - three_deep.trim_start().len();
    let two_deep_indent = two_deep.len() - two_deep.trim_start().len();

    // Verify indentation levels
    assert_eq!(outer_indent, 8, "Outer level should have 8 spaces");
    assert_eq!(nested_base_indent, 12, "Nested level should have 12 spaces");
    assert_eq!(
        nested_extra_indent, 16,
        "Nested with extra should have 16 spaces"
    );
    assert_eq!(two_deep_indent, 12, "Two levels deep should have 12 spaces");
    assert_eq!(
        three_deep_indent, 16,
        "Three levels deep should have 16 spaces"
    );
}

#[test]
fn test_expected_outputs() {
    // Get predefined config mappings
    let configs = Config::test_configs();

    // Find all expected output files
    let examples_dir = std::path::Path::new("examples");
    let expected_files: Vec<_> = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let filename = path.file_name()?.to_str()?;

            // Match pattern: example.[config-key].expected.rs
            if filename.ends_with(".expected.rs") {
                // Extract base name and config key
                let without_expected = filename.strip_suffix(".expected.rs")?;
                let parts: Vec<&str> = without_expected.rsplitn(2, '.').collect();
                if parts.len() == 2 {
                    let config_key = parts[0];
                    let base_name = parts[1];
                    Some((
                        base_name.to_string(),
                        config_key.to_string(),
                        filename.to_string(),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let expected_files_count = expected_files.len();

    for (base_name, config_key, expected_filename) in expected_files {
        // Check if the config key is valid
        let config = match configs.get(config_key.as_str()) {
            Some(cfg) => cfg.clone(),
            None => {
                panic!(
                    "Unknown config key '{}' in file '{}'. Valid keys are: {:?}",
                    config_key,
                    expected_filename,
                    configs.keys().collect::<Vec<_>>()
                );
            }
        };

        // Read the source .vl file
        let source_path = format!("examples/{}.vl", base_name);
        let veltrano_code = match fs::read_to_string(&source_path) {
            Ok(code) => code,
            Err(_) => {
                panic!(
                    "Expected output file '{}' exists but source file '{}' not found",
                    expected_filename, source_path
                );
            }
        };

        // Read the expected output
        let expected_path = format!("examples/{}", expected_filename);
        let expected_rust = fs::read_to_string(&expected_path).expect(&format!(
            "Failed to read expected output file {}",
            expected_path
        ));

        // Transpile the source
        let mut lexer = Lexer::with_config(veltrano_code.clone(), config.clone());
        let all_tokens = lexer.tokenize();
        let mut parser = Parser::new(all_tokens);

        let program = match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                panic!(
                    "Failed to parse {} for config '{}': {}",
                    source_path, config_key, err
                );
            }
        };

        let mut codegen = CodeGenerator::with_config(config);
        let actual_rust = codegen.generate(&program);

        // Compare output (trim to handle trailing newlines)
        if actual_rust.trim() != expected_rust.trim() {
            // For better error messages, show the diff
            eprintln!("\n=== EXPECTED OUTPUT MISMATCH ===");
            eprintln!("File: {}", expected_filename);
            eprintln!("Config: {}", config_key);
            eprintln!("\n--- Expected ---\n{}", expected_rust);
            eprintln!("\n--- Actual ---\n{}", actual_rust);
            eprintln!("\n--- Diff ---");

            // Simple line-by-line diff
            let expected_lines: Vec<&str> = expected_rust.lines().collect();
            let actual_lines: Vec<&str> = actual_rust.lines().collect();
            let max_lines = expected_lines.len().max(actual_lines.len());

            for i in 0..max_lines {
                let expected_line = expected_lines.get(i).unwrap_or(&"<EOF>");
                let actual_line = actual_lines.get(i).unwrap_or(&"<EOF>");

                if expected_line != actual_line {
                    eprintln!("Line {}:", i + 1);
                    eprintln!("  - {}", expected_line);
                    eprintln!("  + {}", actual_line);
                }
            }

            panic!(
                "Output mismatch for {} with config '{}'",
                base_name, config_key
            );
        }
    }

    // Print summary if no expected files found
    if expected_files_count == 0 {
        println!("Note: No expected output files found. Expected files should match pattern: example.[config-key].expected.rs");
    } else {
        println!("Validated {} expected output files", expected_files_count);
    }
}
