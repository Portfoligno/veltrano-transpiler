mod ast;
mod builtins;
mod codegen;
mod comments;
mod config;
mod debug;
mod error;
mod lexer;
mod parser;
mod rust_interop;
mod type_checker;
mod types;

use std::env;
use std::fs;
use std::io::IsTerminal;
use std::process;

use codegen::CodeGenerator;
use config::Config;
use error::ErrorFormatter;
use lexer::Lexer;
use parser::Parser;
use type_checker::VeltranoTypeChecker;

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
    println!("    --fail-fast              Stop at first error instead of collecting all errors");
    println!();
    println!("ARGS:");
    println!("    <input.vl>               The Veltrano source file to transpile");
    println!();
    println!("ENVIRONMENT:");
    println!("    VELTRANO_DEBUG           Set to any value to enable debug output");
    println!();
    println!("EXAMPLES:");
    println!("    {} hello.vl", program_name);
    println!(
        "    {} --preserve-comments examples/fibonacci.vl",
        program_name
    );
    println!("    VELTRANO_DEBUG=1 {} input.vl", program_name);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [OPTIONS] <input.vl>", args[0]);
        process::exit(1);
    }

    let mut preserve_comments = false;
    let mut debug_mode = false;
    let mut fail_fast = false;
    let mut input_file = None;

    // Check if we should use color (default: auto-detect)
    let use_color = std::io::stderr().is_terminal();

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
            "--fail-fast" => {
                fail_fast = true;
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

    // Store a reference to source code for error formatting
    let source_ref = source_code.clone();

    let config = Config { preserve_comments };

    let mut lexer = Lexer::with_config(source_code, config.clone());
    let all_tokens = lexer.tokenize();

    let mut parser = Parser::new(all_tokens);
    let program = if fail_fast {
        match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                let formatter = ErrorFormatter::new(&err, &source_ref)
                    .with_filename(input_file)
                    .with_color(use_color);
                eprintln!("{}", formatter.format());
                process::exit(1);
            }
        }
    } else {
        let (program, errors) = parser.parse_with_recovery();
        if errors.has_errors() {
            eprintln!("Found {} parse error(s):\n", errors.error_count());
            for error in errors.errors() {
                let formatter = ErrorFormatter::new(error, &source_ref).with_filename(input_file);
                eprintln!("{}\n", formatter.format());
            }
            process::exit(1);
        }
        program
    };

    // Type checking phase
    let mut type_checker = VeltranoTypeChecker::new();
    if let Err(errors) = type_checker.check_program_unified(&program) {
        eprintln!("Type checking failed with {} error(s):", errors.len());

        for error in errors {
            eprintln!("  {}", error);
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
    let rust_code = match codegen.generate(&program) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Code generation error: {}", err);
            process::exit(1);
        }
    };

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
