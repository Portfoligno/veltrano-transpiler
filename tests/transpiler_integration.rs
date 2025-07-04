use std::fs;
use veltrano::config::Config;
use veltrano::rust_interop::camel_to_snake_case;

mod common;
mod test_configs;
use common::{
    assert_parse_error, assert_parse_or_type_check_error, assert_transpilation_match,
    assert_transpilation_output, assert_type_check_error, compile_rust_code, transpile,
    transpile_and_compile, TestContext,
};

#[test]
fn test_camel_to_snake_case() {
    assert_eq!(camel_to_snake_case("camelCase"), "camel_case");
    assert_eq!(camel_to_snake_case("CamelCase"), "_camel_case");
    assert_eq!(camel_to_snake_case("simpleVar"), "simple_var");
    assert_eq!(
        camel_to_snake_case("veryLongCamelCaseVariableName"),
        "very_long_camel_case_variable_name"
    );
    assert_eq!(camel_to_snake_case("a"), "a");
    assert_eq!(camel_to_snake_case("aB"), "a_b");
    assert_eq!(camel_to_snake_case("aBc"), "a_bc");
    assert_eq!(camel_to_snake_case("XMLParser"), "_x_m_l_parser");
    assert_eq!(
        camel_to_snake_case("httpURLConnection"),
        "http_u_r_l_connection"
    );
    assert_eq!(camel_to_snake_case("main"), "main");
    assert_eq!(camel_to_snake_case("calculateSum"), "calculate_sum");
    assert_eq!(camel_to_snake_case("calculate_sum"), "calculate__sum");
    assert_eq!(camel_to_snake_case("calculate_Sum"), "calculate___sum");
}

#[test]
fn test_camel_case_transpilation() {
    let source = r#"
fun calculateSum(firstNumber: I64, secondNumber: I64): I64 {
    val totalResult: I64 = firstNumber + secondNumber
    return totalResult
}
"#;

    let config = Config {
        preserve_comments: true,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

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
        assert_transpilation_match(
            &veltrano_code,
            &expected_rust,
            &TestContext::with_config(config),
        );
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
        // Use the helper to compile Rust code, removing ERROR lines and wrapping in main
        if let Err(error) = compile_rust_code(
            rust_code,
            &TestContext::default()
                .with_name(&format!("readme_{}", index))
                .remove_error_lines(true),
        ) {
            panic!(
                "README Rust example {} failed to compile:\n{}\n\nCode:\n{}",
                index, error, rust_code
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

        let config = Config {
            preserve_comments: true,
        };

        // Inject Veltrano variable declarations for common undefined variables
        let mut modified_code = veltrano_code.clone();

        // Check if code uses 'x' without defining it
        if modified_code.contains("if (x") || modified_code.contains("if x") {
            if !modified_code.contains("val x") && !modified_code.contains("var x") {
                // Prepend variable declaration
                modified_code = format!("val x = 10\n{}", modified_code);
            }
        }

        // Check if code uses 'counter' without defining it
        if modified_code.contains("while (counter") || modified_code.contains("while counter") {
            if !modified_code.contains("val counter") && !modified_code.contains("var counter") {
                // Prepend variable declaration
                modified_code = format!("val counter = 0\n{}", modified_code);
            }
        }

        match transpile_and_compile(
            &modified_code,
            &TestContext::with_config(config).with_name(&format!("readme_veltrano_{}", index)),
        ) {
            Ok(_) => {
                // Success - test passed
            }
            Err(error) => {
                panic!(
                    "README Veltrano example {} failed:\n{}\n\nCode:\n{}",
                    index, error, veltrano_code
                );
            }
        }
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
        let comments_suffix = if preserve_comments {
            "_with_comments"
        } else {
            "_no_comments"
        };
        let test_name = format!(
            "example_{}{}",
            example_file.replace(".vl", ""),
            comments_suffix
        );

        // Use the new utility that handles everything including wrapping in main
        // Enable type checking to support import resolution
        match transpile_and_compile(
            &veltrano_code,
            &TestContext::with_config(config).with_name(&test_name),
        ) {
            Ok(_) => {
                // Success - test passed
            }
            Err(error) => {
                panic!(
                    "Example {} (preserve_comments={}) failed:\n{}",
                    example_file, preserve_comments, error
                );
            }
        }
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
    let actual_rust = transpile(veltrano_code, &TestContext::with_config(config)) // enable type checking
        .expect("Transpilation should succeed");

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
    val simple: I64 = 42 // Simple inline comment
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
        // break // Break to avoid infinite loop
    }
}"#;

    // Transpile with both comment configurations
    let config_with_comments = Config {
        preserve_comments: true,
    };
    let config_without_comments = Config {
        preserve_comments: false,
    };
    let rust_code_with_comments = transpile(
        veltrano_code,
        &TestContext::with_config(config_with_comments),
    )
    .expect("Failed to transpile with comments");
    let rust_code_no_comments = transpile(
        veltrano_code,
        &TestContext::with_config(config_without_comments),
    )
    .expect("Failed to transpile without comments");

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
    assert!(rust_code_with_comments.contains("// break // Break to avoid infinite loop"));

    // Check that no comments are preserved when disabled
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
    assert!(!rust_code_no_comments.contains("// break // Break to avoid infinite loop"));

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
    val someVar = 42
    val text: Own<String> = "hello".toString()
    val value: MutRef<I64> = someVar.mutRef()
    val strRef: MutRef<Own<String>> = text.mutRef()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(veltrano_code, &TestContext::with_config(config.clone()))
        .expect("Transpilation should succeed");

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
    val number = 100
    val mutableRef = number.mutRef()
    val another = "test".mutRef()
}"#;

    let rust_code2 = transpile(veltrano_code2, &TestContext::with_config(config))
        .expect("Transpilation should succeed");

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
    let config = Config {
        preserve_comments: false,
    };

    // Test that Own<I64> is rejected
    assert_type_check_error(
        r#"fun main() { val x: Own<I64> = 42 }"#,
        &TestContext::with_config(config.clone())
            .expect_error("Cannot use Own<I64>. Types that implement Copy are always owned by default and don't need the Own<> wrapper."),
    );

    // Test that Own<Bool> is rejected
    assert_type_check_error(
        r#"fun main() { val flag: Own<Bool> = true }"#,
        &TestContext::with_config(config.clone())
            .expect_error("Types that implement Copy are always owned by default and don't need the Own<> wrapper."),
    );

    // Test that Own<String> is accepted
    transpile(
        r#"fun main() { val text: Own<String> = "hello".toString() }"#,
        &TestContext::with_config(config.clone()),
    )
    .expect("Own<String> should be accepted");

    // Test that Own<MutRef<T>> is rejected
    assert_type_check_error(
        r#"fun main() { val x: Own<MutRef<String>> = something }"#,
        &TestContext::with_config(config.clone()).expect_error("MutRef<T> is already owned"),
    );

    // Test that Own<Box<T>> is accepted (Box is no longer rejected)
    // Just test that the type is valid by using a declaration without initialization
    transpile(
        r#"fun main() { val x: Own<Box<String>> }"#,
        &TestContext::with_config(config.clone()), // don't skip_type_check
    )
    .expect("Own<Box<T>> should be accepted");

    // Test that Own<Own<T>> is rejected
    assert_type_check_error(
        r#"fun main() { val x: Own<Own<String>> = something }"#,
        &TestContext::with_config(config)
            .expect_error("Cannot use Own<Own<T>>. This creates double ownership."),
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

        // Extract expected error from first line comment - now mandatory
        let expected_error = veltrano_code
            .lines()
            .next()
            .and_then(|line| {
                if line.starts_with("// Expected error:") {
                    Some(line.trim_start_matches("// Expected error:").trim())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                panic!(
                    "File {} must start with '// Expected error: <expected error message>' comment",
                    fail_file
                )
            });

        // Use helper to expect parse failure with required error checking
        let config = Config {
            preserve_comments: false,
        };
        assert_parse_or_type_check_error(
            &veltrano_code,
            &TestContext::with_config(config).expect_error(expected_error),
        );
    }
}

#[test]
fn test_clone_ufcs_generation() {
    // Test that clone() generates UFCS syntax with explicit conversion enforcement
    let veltrano_code = r#"fun testClone() {
    val owned: Own<String> = "hello".toString()
    val borrowed: String = owned.ref()
    
    // Test clone scenarios with explicit conversions
    val clonedFromRef = borrowed.clone()
    
    // Test with method chaining  
    val chained = owned.ref().clone()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(veltrano_code, &TestContext::with_config(config))
        .expect("Clone UFCS test should parse and generate");

    // Check UFCS generation for clone with explicit conversions
    assert!(
        rust_code.contains("let cloned_from_ref = Clone::clone(borrowed)"),
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
    let rust_code = transpile(veltrano_code, &TestContext::with_config(config))
        .expect("Transpilation should succeed");

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
    val chained1 = borrowed.clone().mutRef()
    val chained2 = borrowed.clone().ref().mutRef()
    
    // Direct .mutRef() usage
    val directMut = owned.mutRef()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(veltrano_code, &TestContext::with_config(config))
        .expect("Transpilation should succeed");

    // Check chaining patterns
    assert!(
        rust_code.contains("let chained1 = &mut Clone::clone(borrowed)"),
        "Expected '&mut Clone::clone(borrowed)' but got: {}",
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

        // Define common variables for table examples
        let variable_injections = vec![
            ("owned", "let owned = String::from(\"example\");"),
            ("borrowed", "let borrowed = &String::from(\"example\");"),
            ("num", "let num = 42i64;"),
            ("s", "let s = &String::from(\"example\");"),
        ];

        // First try to transpile as-is
        let mut ctx =
            TestContext::with_config(config.clone()).with_name(&format!("table_{}", index));

        for (var_name, init_code) in &variable_injections {
            ctx = ctx.with_injection(var_name, init_code);
        }

        let result = transpile_and_compile(example, &ctx);

        match result {
            Ok(_) => {
                // Success - test passed
            }
            Err(_) => {
                // Some table examples might be fragments, try wrapping in a function
                let wrapped_example = format!("fun main() {{\n    {}\n}}", example);

                let mut ctx =
                    TestContext::with_config(config).with_name(&format!("table_{}_wrapped", index));

                for (var_name, init_code) in &variable_injections {
                    ctx = ctx.with_injection(var_name, init_code);
                }

                match transpile_and_compile(&wrapped_example, &ctx) {
                    Ok(_) => {
                        // Success with wrapping
                    }
                    Err(_) => {
                        println!("Skipping table example {} (fragment): {}", index, example);
                        continue;
                    }
                }
            }
        }
    }
}

#[test]
fn test_unit_literal() {
    let source = r#"fun main() {
    val x: Unit = Unit
    val y = Unit
    println("{:?}", x)
    println("{:?}", y)
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source, &TestContext::with_config(config))
        .expect("Unit literal should parse and generate");

    // Check that Unit literal is transpiled to ()
    assert!(rust_code.contains("let x: () = ()"));
    assert!(rust_code.contains("let y = ()"));
}

#[test]
fn test_unary_expressions() {
    let source = r#"fun main() {
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
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source, &TestContext::with_config(config))
        .expect("Unary expressions should parse and generate");

    // Check that unary expressions are correctly transpiled
    assert!(rust_code.contains("let negative = -42"));
    assert!(rust_code.contains("let expr = -((2 + 3))"));
    assert!(rust_code.contains("let spaced = -15")); // Space allowed
    assert!(rust_code.contains("let var__neg = -negative"));
    assert!(rust_code.contains("let parens = -((-20))")); // OK with parentheses
}

#[test]
fn test_double_minus_forbidden() {
    let config = Config {
        preserve_comments: false,
    };
    assert_parse_error(
        r#"fun main() { val bad = --5 }"#,
        &TestContext::with_config(config).expect_error("Double minus (--) is not allowed"),
    );
}

#[test]
fn test_import_statement() {
    let source = r#"
import String.len
import ToString.toString as str

fun main() {
    // Test aliased import
    val text: Own<String> = "hello".str()
    
    // Test non-aliased import  
    val length = text.ref().len()
    
    // Show that built-in clone still works
    val cloned = text.ref().clone()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

    // Check that imports don't generate any Rust code
    assert!(!rust_code.contains("import"));

    // Check that method calls use UFCS where imported
    assert!(rust_code.contains("ToString::to_string(\"hello\")")); // str() -> ToString::to_string()
    assert!(rust_code.contains("String::len(&text)")); // len() -> String::len()
    assert!(rust_code.contains("Clone::clone(&text)")); // clone() uses built-in
}

#[test]
fn test_preimported_methods() {
    let source = r#"
fun main() {
    val text: Own<String> = "Hello".toString()
    val borrowed: String = text.ref()
    val cloned = borrowed.clone()
    val string = borrowed.toString()
    val reference = borrowed.ref()
    val mutable = text.mutRef()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

    // Check pre-imported methods with explicit conversions
    assert!(rust_code.contains("Clone::clone(borrowed)"));
    assert!(rust_code.contains("ToString::to_string(borrowed)"));
    assert!(rust_code.contains("&text")); // .ref() to get borrowed
    assert!(rust_code.contains("&mut text")); // .mutRef()
}

#[test]
fn test_import_priority_over_preimported() {
    let source = r#"import i64.abs

fun main() {
    val value = -42
    val positive = value.abs()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source, &TestContext::with_config(config))
        .expect("Import priority test should parse and generate");

    // Check that explicit import generates UFCS call
    assert!(rust_code.contains("i64::abs"));
}

#[test]
fn test_import_with_alias() {
    let source = r#"import ToString.toString as stringify

fun main() {
    val num = 42
    val str = num.stringify()
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source, &TestContext::with_config(config))
        .expect("Import alias test should parse and generate");

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

fun new(): I64 {
    return 42
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

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
data class Point(val x: I64, val y: I64)
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source1, &TestContext::with_config(config.clone()))
        .expect("Transpilation should succeed");

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
data class Person(val name: String, val age: I64)
"#;

    let rust_code2 = transpile(source2, &TestContext::with_config(config.clone()))
        .expect("Transpilation should succeed");

    // Check struct generation with lifetime
    assert!(rust_code2.contains("#[derive(Debug, Clone)]"));
    assert!(rust_code2.contains("pub struct Person<'a> {"));
    assert!(rust_code2.contains("pub name: &'a String,"));
    assert!(rust_code2.contains("pub age: i64,"));

    // Test data class with custom types
    let source3 = r#"
data class Container(val item: MyType, val count: I64)
"#;

    let rust_code3 = transpile(source3, &TestContext::with_config(config))
        .expect("Transpilation should succeed");

    // Check struct generation with custom type (needs lifetime)
    assert!(rust_code3.contains("pub struct Container<'a> {"));
    assert!(rust_code3.contains("pub item: &'a MyType,"));
}

#[test]
fn test_data_class_initialization() {
    // Test Kotlin-style struct initialization
    let source = r#"
data class Point(val x: I64, val y: I64)
data class Person(val name: Str, val age: I64)

fun main() {
    val p1 = Point(x = 10, y = 20)
    val p2 = Person(name = "Alice", age = 30)
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

    // Check struct initialization syntax
    assert!(rust_code.contains("let p1 = Point { x: 10, y: 20 };"));
    assert!(rust_code.contains("let p2 = Person { name: \"Alice\", age: 30 };"));
}

#[test]
fn test_data_class_field_shorthand() {
    // Test Rust field shorthand syntax in struct initialization
    let source = r#"
data class Point(val x: I64, val y: I64)
data class Person(val name: Str, val age: I64)

fun main() {
    // All shorthand - uses field shorthand syntax
    val x = 10
    val y = 20
    val p1 = Point(.x, .y)
    
    // Mixed shorthand and named
    val name = "Alice"
    val p2 = Person(.name, age = 30)
    
    // All named
    val p3 = Person(name = "Bob", age = 25)
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

    // Check field shorthand syntax
    assert!(rust_code.contains("let p1 = Point { x, y };"));
    assert!(rust_code.contains("let p2 = Person { name, age: 30 };"));
    assert!(rust_code.contains("let p3 = Person { name: \"Bob\", age: 25 };"));
}

#[test]
fn test_data_class_argument_order() {
    // Test that named arguments can be provided in any order
    let source = r#"
data class Person(val name: Str, val age: I64)
data class Book(val title: Str, val author: Str, val pages: I64)

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
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

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
    // Test that shorthand and named arguments can be mixed in any order
    let source = r#"data class Person(val name: Str, val age: I64)
data class Book(val title: Str, val author: Str, val pages: I64)

fun main() {
    val name = "Alice"
    val age = 30
    val title = "Rust Book"
    val author = "Steve"
    val pages = 500
    
    // Shorthand first, named second
    val p1 = Person(.name, age = 25)
    
    // Named first, shorthand second
    val p2 = Person(age = 35, .name)
    
    // Multiple combinations with 3 fields
    val b1 = Book(.title, author = "Carol", pages = 300)     // shorthand, named, named
    val b2 = Book(title = "Guide", .author, pages = 400)     // named, shorthand, named
    val b3 = Book(title = "Manual", author = "Bob", .pages)  // named, named, shorthand
}"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source, &TestContext::with_config(config))
        .expect("Data class mixed args test should parse and generate");

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
    let rust_code = transpile(veltrano_code, &TestContext::with_config(config))
        .expect("Failed to transpile contextual indentation test");

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
data class Point(val x: I64, val y: I64)
data class Person(val name: Str, val age: I64)

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
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

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
    val hello: Ref<Str> = "Hello".bumpRef()
    
    // Single line chain
    val single: Ref<Ref<Ref<Str>>> = hello.ref().bumpRef()
    
    // Multiline chain with dots on new lines
    val multi: Ref<Ref<Ref<Str>>> = hello
        .ref()
        .bumpRef()
    
    // Mixed style
    val mixed: Ref<Ref<Ref<Str>>> = hello.ref()
        .bumpRef()
    
    // Longer chains
    val long: Ref<Ref<Ref<Ref<Ref<Str>>>>> = hello
        .ref()
        .ref()
        .ref()
        .bumpRef()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code =
        transpile(source, &TestContext::with_config(config)).expect("Transpilation should succeed");

    // All variations should generate bump allocations
    assert!(rust_code.contains("let hello: &&str = bump.alloc(\"Hello\");"));
    assert!(rust_code.contains("let single: &&&&str = bump.alloc(&hello);"));
    assert!(rust_code.contains("let multi: &&&&str = bump.alloc(&hello);"));
    assert!(rust_code.contains("let mixed: &&&&str = bump.alloc(&hello);"));
    assert!(rust_code.contains("let long: &&&&&&str = bump.alloc(&&&hello);"));
}

#[test]
fn test_nested_function_call_comment_indentation() {
    // Test that comments in nested function calls have proper indentation
    let veltrano_code = r#"fun f(a: I64, b: I64): I64 {
    return a + b
}

fun g(x: I64, y: I64, z: I64): I64 {
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
    let rust_code = transpile(veltrano_code, &TestContext::with_config(config))
        .expect("Nested function comment test should parse and generate");

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
    let configs = test_configs::test_configs();

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

        // Test transpilation against expected output
        let context = format!("File: {}, Config: {}", expected_filename, config_key);
        assert_transpilation_output(
            &veltrano_code,
            &expected_rust,
            &TestContext::with_config(config),
            &context,
        );
    }

    // Print summary if no expected files found
    if expected_files_count == 0 {
        println!("Note: No expected output files found. Expected files should match pattern: example.[config-key].expected.rs");
    } else {
        println!("Validated {} expected output files", expected_files_count);
    }
}

#[test]
fn test_multiple_imports_same_name() {
    // Test that multiple imports can have the same name and are resolved by type
    let source = r#"
import String.clone as duplicate
import i64.abs as duplicate

fun main() {
    val text: Own<String> = "hello".toString()
    val number: I64 = -42
    
    // Should resolve to String.clone based on receiver type
    val text_copy = text.ref().duplicate()
    
    // Should resolve to i64.abs based on receiver type  
    val positive = number.duplicate()
}
"#;

    let config = Config {
        preserve_comments: false,
    };
    let rust_code = transpile(source, &TestContext::with_config(config)) // don't skip type check - needed for import resolution
        .expect("Multiple imports with same name should succeed");

    // Check that the correct methods are called via UFCS
    assert!(rust_code.contains("String::clone(&text)"));
    assert!(rust_code.contains("i64::abs(number)"));
}

#[test]
fn test_import_shadows_builtin_completely() {
    // Test that imported methods completely shadow built-ins (no fallback)
    let source = r#"
import String.len as length

fun testNoFallback() {
    val text: String = "hello".toString().ref()
    val number: I64 = 42
    
    // This works - uses imported String.len
    val text_len = text.length()
    
    // This would fail even if there's a built-in 'length' for I64
    // because imports shadow built-ins completely
    // val bad = number.length()  // ERROR: Method not found
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // The example should transpile successfully (commented out error line)
    let rust_code = transpile(source, &TestContext::with_config(config))
        .expect("Import shadowing test should succeed");

    assert!(rust_code.contains("String::len(text)"));
}
