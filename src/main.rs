mod ast;
mod codegen;
mod config;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::process;

use codegen::CodeGenerator;
use config::Config;
use lexer::Lexer;
use parser::Parser;

fn print_help(program_name: &str) {
    println!("Veltrano Transpiler v{}", env!("CARGO_PKG_VERSION"));
    println!("A transpiler for the Veltrano programming language");
    println!();
    println!("USAGE:");
    println!("    {} <input.vl>", program_name);
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -v, --version    Print version information");
    println!();
    println!("ARGS:");
    println!("    <input.vl>       The Veltrano source file to transpile");
    println!();
    println!("EXAMPLES:");
    println!("    {} hello.vl", program_name);
    println!("    {} examples/fibonacci.vl", program_name);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.vl>", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];

    // Handle --version flag
    if input_file == "--version" || input_file == "-v" {
        println!("veltrano {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    // Handle --help flag
    if input_file == "--help" || input_file == "-h" {
        print_help(&args[0]);
        process::exit(0);
    }

    let source_code = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", input_file, err);
            process::exit(1);
        }
    };

    let config = Config {
        preserve_comments: false, // Set to true to preserve comments in output
    };

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

    let mut codegen = CodeGenerator::with_config(config);
    let rust_code = codegen.generate(&program);

    let output_file = input_file.replace(".vl", ".rs");

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
