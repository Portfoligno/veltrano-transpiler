mod ast;
mod codegen;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::process;

use codegen::CodeGenerator;
use lexer::{Lexer, skip_comments};
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

    let mut lexer = Lexer::new(source_code);
    let all_tokens = lexer.tokenize();
    let tokens = skip_comments(all_tokens);

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            process::exit(1);
        }
    };

    let mut codegen = CodeGenerator::new();
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
