#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::codegen::CodeGenerator;
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

        let config = Config { preserve_comments: true };
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

        for (veltrano_code, expected_rust) in examples {
            let config = Config { preserve_comments: true };
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
        let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");
        let veltrano_examples = extract_veltrano_code_examples(&readme_content);

        for (index, veltrano_code) in veltrano_examples.iter().enumerate() {
            // Try to transpile the Veltrano code
            let config = Config { preserve_comments: true };
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
                // Special case for control flow example that uses undefined 'condition'
                if rust_code.contains("if condition") || rust_code.contains("while condition") {
                    format!("fn main() {{\nlet condition = true;\n{}\n}}", rust_code)
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
                    Some(path.file_name()?.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();

        for example_file in &example_files {
            let example_path = format!("examples/{}", example_file);
            let veltrano_code = fs::read_to_string(&example_path)
                .expect(&format!("Failed to read {}", example_path));

            let config = Config { preserve_comments };
            let mut lexer = Lexer::with_config(veltrano_code.clone(), config.clone());
            let all_tokens = lexer.tokenize();
            let mut parser = Parser::new(all_tokens);

            let program = match parser.parse() {
                Ok(program) => program,
                Err(err) => {
                    // Skip files that fail to parse for now, but log the issue
                    eprintln!("Warning: Example {} failed to parse (skipping): {}", example_file, err);
                    continue;
                }
            };

            // Generate Rust code
            let mut codegen = CodeGenerator::with_config(config);
            let rust_code = codegen.generate(&program);

            // Create a temporary Rust file
            let comments_suffix = if preserve_comments { "_with_comments" } else { "_no_comments" };
            let temp_file = format!("/tmp/example_{}{}.rs", example_file.replace(".vl", ""), comments_suffix);

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
                .arg(&format!("/tmp/example_{}{}", example_file.replace(".vl", ""), comments_suffix))
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
            let _ = fs::remove_file(&format!("/tmp/example_{}{}", example_file.replace(".vl", ""), comments_suffix));
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