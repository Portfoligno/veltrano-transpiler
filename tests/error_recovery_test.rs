//! Tests for error recovery and reporting

use veltrano::{Config, Lexer, Parser};

#[test]
fn test_parser_error_recovery() {
    let source = r#"
fun valid1() {
    println("First valid function")
}

fun missing_type(x) {  // Error: missing type
    x + 1
}

fun valid2() {
    println("Second valid function")
}

fun missing_body()  // Error: missing body

fun valid3() {
    println("Third valid function")
}
"#;

    let mut lexer = Lexer::with_config(source.to_string(), Config::default());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, errors) = parser.parse_with_recovery();

    // Should have parsed at least 2 valid functions despite errors
    let function_count = program
        .statements
        .iter()
        .filter(|stmt| matches!(stmt, veltrano::ast::Stmt::FunDecl(_)))
        .count();
    assert!(
        function_count >= 2,
        "Should have parsed at least 2 valid functions, got {}",
        function_count
    );

    // Should have found errors
    assert!(errors.has_errors(), "Should have found errors");
    assert!(errors.error_count() >= 2, "Should have at least 2 errors");
}

#[test]
fn test_multiple_parameter_errors() {
    let source = r#"
fun test(a, b: i32, c) -> i32 {
    a + b + c
}
"#;

    let mut lexer = Lexer::with_config(source.to_string(), Config::default());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, errors) = parser.parse_with_recovery();

    // Should report missing types for both 'a' and 'c'
    assert!(
        errors.error_count() >= 2,
        "Should find missing type errors for 'a' and 'c'"
    );
}

#[test]
fn test_error_with_help_messages() {
    let source = r#"fun test(x) { x + 1 }"#;

    let mut lexer = Lexer::with_config(source.to_string(), Config::default());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, errors) = parser.parse_with_recovery();

    // Check that error has helpful context
    let first_error = errors.errors().first().unwrap();
    assert!(
        first_error.context.note.is_some(),
        "Error should have a note"
    );
    assert!(
        first_error.context.help.is_some(),
        "Error should have help text"
    );
}

#[test]
fn test_error_location_accuracy() {
    let source = r#"fun test(x: i32, y) { x + y }"#;

    let mut lexer = Lexer::with_config(source.to_string(), Config::default());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, errors) = parser.parse_with_recovery();

    // Check error location points to 'y' parameter
    let error = errors.errors().first().unwrap();
    if let Some(span) = &error.context.span {
        assert_eq!(span.start.column, 19, "Error should point to parameter 'y'");
    } else {
        panic!("Error should have a span");
    }
}

#[test]
fn test_no_errors_with_valid_code() {
    let source = r#"
fun add(x: I64, y: I64): I64 {
    return x + y
}

fun main() {
    println("Hello")
}
"#;

    let mut lexer = Lexer::with_config(source.to_string(), Config::default());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, errors) = parser.parse_with_recovery();

    assert!(!errors.has_errors(), "Valid code should have no errors");
    assert_eq!(program.statements.len(), 2, "Should parse both functions");
}
