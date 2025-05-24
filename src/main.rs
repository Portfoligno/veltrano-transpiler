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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.vl>", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];

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

    let mut lexer = Lexer::new(source_code);
    let all_tokens = lexer.tokenize();

    let mut parser = Parser::with_config(all_tokens, &config);
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
