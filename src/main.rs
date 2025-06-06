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

    let mut codegen = CodeGenerator::with_config(config);
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
