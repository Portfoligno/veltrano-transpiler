mod ast;
mod builtins;
mod codegen;
mod config;
mod debug;
mod lexer;
mod parser;
mod rust_interop;
mod type_checker;
mod types;

use std::env;
use std::fs;
use std::process;

use codegen::CodeGenerator;
use config::Config;
use lexer::Lexer;
use parser::Parser;
use type_checker::{ErrorAnalyzer, TypeCheckError, VeltranoTypeChecker};

fn format_type_error(error: &TypeCheckError) -> String {
    match error {
        TypeCheckError::TypeMismatch {
            expected,
            actual,
            location,
        } => {
            format!(
                "Type mismatch at {}:{}: expected {:?}, found {:?}",
                location.file, location.line, expected, actual
            )
        }
        TypeCheckError::TypeMismatchWithSuggestion {
            expected,
            actual,
            location,
            suggestion,
        } => {
            format!(
                "Type mismatch at {}:{}: expected {:?}, found {:?}. Try: {}",
                location.file, location.line, expected, actual, suggestion
            )
        }
        TypeCheckError::MethodNotFound {
            receiver_type,
            method,
            location,
        } => {
            format!(
                "Method '{}' not found on type {:?} at {}:{}",
                method, receiver_type, location.file, location.line
            )
        }
        TypeCheckError::MethodNotFoundWithSuggestion {
            receiver_type,
            method,
            location,
            suggestion,
        } => {
            format!(
                "Method '{}' not found on type {:?} at {}:{}. {}",
                method, receiver_type, location.file, location.line, suggestion
            )
        }
        TypeCheckError::FieldNotFound {
            object_type,
            field,
            location,
        } => {
            format!(
                "Field '{}' not found on type {:?} at {}:{}",
                field, object_type, location.file, location.line
            )
        }
        TypeCheckError::FieldNotFoundWithSuggestion {
            object_type,
            field,
            location,
            suggestion,
        } => {
            format!(
                "Field '{}' not found on type {:?} at {}:{}. {}",
                field, object_type, location.file, location.line, suggestion
            )
        }
        TypeCheckError::ArgumentCountMismatch {
            function,
            expected,
            actual,
            location,
        } => {
            format!(
                "Function '{}' expects {} arguments, but {} were provided at {}:{}",
                function, expected, actual, location.file, location.line
            )
        }
        TypeCheckError::_IndexingNotSupported {
            object_type,
            index_type,
            location,
        } => {
            format!(
                "Indexing not supported: cannot index type {:?} with type {:?} at {}:{}",
                object_type, index_type, location.file, location.line
            )
        }
        TypeCheckError::_BinaryOperatorNotSupported {
            operator,
            left_type,
            right_type,
            location,
        } => {
            format!(
                "Binary operator {:?} not supported for types {:?} and {:?} at {}:{}",
                operator, left_type, right_type, location.file, location.line
            )
        }
        TypeCheckError::VariableNotFound { name, location } => {
            format!(
                "Variable '{}' not found at {}:{}",
                name, location.file, location.line
            )
        }
        TypeCheckError::FunctionNotFound { name, location } => {
            format!(
                "Function '{}' not found at {}:{}",
                name, location.file, location.line
            )
        }
        TypeCheckError::InvalidTypeConstructor { message, location } => {
            format!(
                "Invalid type constructor at {}:{}: {}",
                location.file, location.line, message
            )
        }
        TypeCheckError::UnsupportedFeature { feature, location } => {
            format!(
                "Unsupported feature at {}:{}: {}",
                location.file, location.line, feature
            )
        }
        TypeCheckError::_InvalidType {
            type_name,
            reason,
            location,
        } => {
            format!(
                "Invalid type '{}' at {}:{}: {}",
                type_name, location.file, location.line, reason
            )
        }
        TypeCheckError::_InvalidImport {
            type_name,
            method_name,
            location,
        } => {
            format!(
                "Invalid import: method '{}' not found on type '{}' at {}:{}",
                method_name, type_name, location.file, location.line
            )
        }
        TypeCheckError::AmbiguousMethodCall {
            method,
            receiver_type,
            candidates,
            location,
        } => {
            format!(
                "Ambiguous method call '{}' on type {:?} at {}:{}. Multiple imported methods match: {}",
                method, receiver_type, location.file, location.line, candidates.join(", ")
            )
        }
    }
}

fn print_help(program_name: &str) {
    println!("Veltrano Transpiler {}", env!("CARGO_PKG_VERSION"));
    println!("A transpiler for the Veltrano programming language");
    println!();
    println!("USAGE:");
    println!("    {} [OPTIONS] <input.vl>", program_name);
    println!();
    println!("OPTIONS:");
    println!("    -h, --help               Print help information");
    println!("    -v, --version            Print version information");
    println!("    --preserve-comments      Preserve comments in generated Rust code");
    println!("    --debug                  Enable debug output for troubleshooting");
    println!();
    println!("ARGS:");
    println!("    <input.vl>               The Veltrano source file to transpile");
    println!();
    println!("EXAMPLES:");
    println!("    {} hello.vl", program_name);
    println!(
        "    {} --preserve-comments examples/fibonacci.vl",
        program_name
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [OPTIONS] <input.vl>", args[0]);
        process::exit(1);
    }

    let mut preserve_comments = false;
    let mut debug_mode = false;
    let mut input_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--version" | "-v" => {
                println!("veltrano {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "--help" | "-h" => {
                print_help(&args[0]);
                process::exit(0);
            }
            "--preserve-comments" => {
                preserve_comments = true;
                i += 1;
            }
            "--debug" => {
                debug_mode = true;
                i += 1;
            }
            _ => {
                if input_file.is_none() {
                    input_file = Some(&args[i]);
                    i += 1;
                } else {
                    eprintln!("Unexpected argument: {}", args[i]);
                    eprintln!("Usage: {} [OPTIONS] <input.vl>", args[0]);
                    process::exit(1);
                }
            }
        }
    }

    let input_file = match input_file {
        Some(file) => file,
        None => {
            eprintln!("Missing input file");
            eprintln!("Usage: {} [OPTIONS] <input.vl>", args[0]);
            process::exit(1);
        }
    };

    // Enable debug mode if requested
    if debug_mode {
        debug::enable_debug();
    }

    let source_code = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", input_file, err);
            process::exit(1);
        }
    };

    let config = Config { preserve_comments };

    let mut lexer = Lexer::with_config(source_code, config.clone());
    let all_tokens = lexer.tokenize();

    let mut parser = Parser::new(all_tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            process::exit(1);
        }
    };

    // Type checking phase
    let mut type_checker = VeltranoTypeChecker::new();
    if let Err(errors) = type_checker.check_program(&program) {
        eprintln!("Type checking failed with {} error(s):", errors.len());

        let analyzer = ErrorAnalyzer;
        for error in errors {
            let enhanced_error = analyzer.enhance_error(error);
            eprintln!("  {}", format_type_error(&enhanced_error));
        }
        process::exit(1);
    }

    let mut codegen = CodeGenerator::with_config(config);
    // Pass method resolutions from type checker to codegen
    let resolutions = type_checker.get_method_resolutions().clone();
    veltrano::debug_println!(
        "DEBUG main: Passing {} method resolutions to codegen",
        resolutions.len()
    );
    for (id, res) in &resolutions {
        veltrano::debug_println!("  ID {}: {:?}.{}", id, res.rust_type, res.method_name);
    }
    codegen.set_method_resolutions(resolutions);
    let rust_code = codegen.generate(&program);

    let output_file = if input_file.ends_with(".vl") {
        format!("{}.rs", &input_file[..input_file.len() - 3])
    } else {
        format!("{}.rs", input_file)
    };

    match fs::write(&output_file, rust_code) {
        Ok(_) => println!(
            "Successfully transpiled '{}' to '{}'",
            input_file, output_file
        ),
        Err(err) => {
            eprintln!("Error writing output file '{}': {}", output_file, err);
            process::exit(1);
        }
    }
}
