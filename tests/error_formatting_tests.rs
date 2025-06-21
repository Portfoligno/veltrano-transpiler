//! Tests for error message formatting and consistency

mod common;

use common::snapshot_utils::assert_error_snapshot;
use veltrano::error::{ErrorKind, VeltranoError, SourceLocation, Span};
use veltrano::lexer::Lexer;
use veltrano::parser::Parser;
use veltrano::config::Config;

#[test]
fn test_error_with_context() {
    // Create an error with full context information
    let error = VeltranoError::new(
        ErrorKind::ParseError,
        "Expected closing parenthesis",
    )
    .with_span(Span::single(SourceLocation::new(3, 15)))
    .with_note("Opening parenthesis was here")
    .with_help("Add a closing ')' to match the opening '('");
    
    assert_error_snapshot("error_with_full_context", &error);
}

#[test]
fn test_multiline_error_span() {
    let code = r#"
fun test(
    x: I64,
    y: String
    z: Bool
) {
    return x + y
}
"#;
    
    let config = Config { preserve_comments: false };
    let mut lexer = Lexer::with_config(code.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(_) => panic!("Expected parse error"),
        Err(e) => assert_error_snapshot("multiline_error_span", &e),
    }
}

#[test]
fn test_error_formatting_consistency() {
    // Test that similar errors have consistent formatting
    let errors = vec![
        ("parse_error", ErrorKind::ParseError, "Unexpected token 'if'"),
        ("type_error", ErrorKind::TypeError, "Cannot assign String to I64"),
        ("syntax_error", ErrorKind::SyntaxError, "Invalid character '#'"),
    ];
    
    for (name, kind, message) in errors {
        let error = VeltranoError::new(kind, message)
            .with_span(Span::single(SourceLocation::new(1, 1)));
        assert_error_snapshot(&format!("consistent_formatting_{}", name), &error);
    }
}

#[test]
fn test_nested_expression_error() {
    let code = "val x = (1 + (2 * (3 / )))";
    
    let config = Config { preserve_comments: false };
    let mut lexer = Lexer::with_config(code.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(_) => panic!("Expected parse error in nested expression"),
        Err(e) => assert_error_snapshot("nested_expression_error", &e),
    }
}

#[test]
fn test_error_column_accuracy() {
    // Test that column numbers are accurate for various positions
    let test_cases = vec![
        ("val = 42", "error_column_start"),
        ("val x = ", "error_column_end"),
        ("val x = 1 + + 2", "error_column_middle"),
    ];
    
    for (code, snapshot_name) in test_cases {
        let config = Config { preserve_comments: false };
        let mut lexer = Lexer::with_config(code.to_string(), config);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        
        match parser.parse() {
            Ok(_) => panic!("Expected parse error for: {}", code),
            Err(e) => assert_error_snapshot(snapshot_name, &e),
        }
    }
}
