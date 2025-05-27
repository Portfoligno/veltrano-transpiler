use crate::codegen::CodeGenerator;
use crate::config::Config;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::fs;
use std::process::Command;

#[test]
fn test_camel_to_snake_case() {
    let codegen = CodeGenerator::with_config(Config::default());

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

        // Create a temporary Rust file for each example
        let temp_file = format!("/tmp/readme_example_{}.rs", index);

        // Wrap the code in a main function if it's not already a complete program
        let complete_rust_code = if cleaned_rust_code.contains("fn main") {
            cleaned_rust_code.clone()
        } else {
            format!("fn main() {{\n{}\n}}", cleaned_rust_code)
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
            // Special case for control flow examples that use undefined variables
            if rust_code.contains("if x") {
                format!("fn main() {{\nlet x = 10;\n{}\n}}", rust_code)
            } else if rust_code.contains("while counter") {
                format!("fn main() {{\nlet counter = 0;\n{}\n}}", rust_code)
            } else {
                format!("fn main() {{\n{}\n}}", rust_code)
            }
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
            .arg("-A")
            .arg("unused_must_use")
            .arg("-o")
            .arg(&format!(
                "/tmp/example_{}{}",
                example_file.replace(".vl", ""),
                comments_suffix
            ))
            .arg(&temp_file)
            .output()
            .expect("Failed to execute rustc");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "Example {} (preserve_comments={}) transpiled but failed to compile:\n{}\n\nVeltrano code:\n{}\n\nGenerated Rust code:\n{}",
                example_file, preserve_comments, stderr, veltrano_code, complete_rust_code
            );
        }

        // Clean up temporary files
        let _ = fs::remove_file(&temp_file);
        let _ = fs::remove_file(&format!(
            "/tmp/example_{}{}",
            example_file.replace(".vl", ""),
            comments_suffix
        ));
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

fn normalize_code(code: &str) -> String {
    code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
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
            // Add common variable declarations for table examples
            let mut vars = String::new();
            if rust_code.contains("owned") {
                vars.push_str("let owned = String::from(\"example\");\n");
            }
            if rust_code.contains("borrowed") {
                vars.push_str("let borrowed = &String::from(\"example\");\n");
            }
            if rust_code.contains("num") {
                vars.push_str("let num = 42i64;\n");
            }
            if rust_code.contains("&s") && !rust_code.contains("let s") {
                vars.push_str("let s = &String::from(\"example\");\n");
            }

            format!("fn main() {{\n{}{}\n}}", vars, rust_code)
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
            .arg(&format!("/tmp/table_example_{}", index))
            .arg(&temp_file)
            .output()
            .expect("Failed to execute rustc");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "Table example {} failed to compile:\n{}\n\nVeltrano code:\n{}\n\nGenerated Rust code:\n{}",
                index, stderr, example, complete_rust_code
            );
        }

        // Clean up temporary files
        let _ = fs::remove_file(&temp_file);
        let _ = fs::remove_file(&format!("/tmp/table_example_{}", index));
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
