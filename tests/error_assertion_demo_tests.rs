//! Demonstrates usage of error assertion utilities

mod common;

use common::error_assertions::*;
use veltrano::config::Config;
use veltrano::error::ErrorKind;

#[test]
fn test_parse_error_with_basic_assertions() {
    // Simple assertion that parsing fails
    assert_parse_fails("val x =", ErrorKind::UnexpectedEof, "Expected expression");
}

#[test]
fn test_parse_error_at_location() {
    // Assert parsing fails at specific location
    assert_parse_fails_at(
        "val x = 1 +",
        ErrorKind::UnexpectedEof,
        1,
        12, // line 1, column 12
    );
}

#[test]
fn test_type_error_with_message() {
    // Assert type checking fails with specific message
    assert_type_check_fails(
        "val x: String = 42",
        ErrorKind::TypeMismatch,
        "expected VeltranoType { constructor: String",
    );
}

#[test]
fn test_fluent_error_assertions() {
    use common::parse_veltrano_code;

    // Test parse error with fluent API
    let parse_result = parse_veltrano_code(
        "val = 42",
        Config {
            preserve_comments: false,
        },
    );
    match parse_result {
        Ok(_) => panic!("Expected parse error"),
        Err(error) => {
            ErrorAssertion::assert_that(&error)
                .is_parse_error()
                .has_kind(ErrorKind::SyntaxError)
                .has_message_containing("Expected variable name")
                .at_line(1)
                .at_column(5);
        }
    }
}

#[test]
fn test_chained_type_error_assertions() {
    use common::parse_and_type_check;

    let code = r#"
        val x = "hello"
        val y = x + 42
    "#;

    match parse_and_type_check(
        code,
        Config {
            preserve_comments: false,
        },
    ) {
        Ok(_) => panic!("Expected type error"),
        Err(error) => {
            ErrorAssertion::assert_that(&error)
                .is_type_error()
                .has_kind(ErrorKind::TypeMismatch)
                .at_line(3);
        }
    }
}

#[test]
fn test_undefined_variable_error() {
    assert_type_check_fails(
        "val x = undefined_var",
        ErrorKind::UndefinedVariable,
        "Variable 'undefined_var' not found",
    );
}

#[test]
fn test_undefined_function_error() {
    assert_type_check_fails(
        "val x = unknown_func()",
        ErrorKind::UndefinedFunction,
        "Function 'unknown_func' not found",
    );
}

#[test]
fn test_invalid_method_call_error() {
    assert_type_check_fails(
        r#"val s = "hello"
val x = s.unknown_method()"#,
        ErrorKind::InvalidMethodCall,
        "Method 'unknown_method' not found",
    );
}

#[test]
fn test_double_minus_syntax_error() {
    use common::parse_veltrano_code;

    let result = parse_veltrano_code(
        "val x = --5",
        Config {
            preserve_comments: false,
        },
    );
    match result {
        Ok(_) => panic!("Expected syntax error"),
        Err(error) => {
            ErrorAssertion::assert_that(&error)
                .has_kind(ErrorKind::SyntaxError)
                .has_message_containing("Double minus (--) is not allowed")
                .at_location(1, 10);
        }
    }
}

#[test]
fn test_error_with_help_and_note() {
    use veltrano::error::{SourceLocation, Span, VeltranoError};

    // Create an error with help and note
    let error = VeltranoError::new(ErrorKind::ParseError, "Missing closing parenthesis")
        .with_span(Span::single(SourceLocation::new(2, 15)))
        .with_note("Opening parenthesis was on line 1")
        .with_help("Add a ')' to close the expression");

    ErrorAssertion::assert_that(&error)
        .has_kind(ErrorKind::ParseError)
        .has_message("Missing closing parenthesis")
        .at_location(2, 15)
        .has_note_containing("Opening parenthesis")
        .has_help_containing("Add a ')'");
}

// Test the assert_error! macro
#[test]
fn test_assert_error_macro_simple() {
    use common::parse_veltrano_code;

    let result = parse_veltrano_code(
        "val x =",
        Config {
            preserve_comments: false,
        },
    );
    assert_error!(result, ErrorKind::UnexpectedEof);
}

#[test]
fn test_assert_error_macro_with_message() {
    use common::parse_veltrano_code;

    let result = parse_veltrano_code(
        "val x =",
        Config {
            preserve_comments: false,
        },
    );
    assert_error!(result, ErrorKind::UnexpectedEof, "Expected expression");
}

#[test]
fn test_assert_error_macro_with_location() {
    use common::parse_veltrano_code;

    let result = parse_veltrano_code(
        "val x =",
        Config {
            preserve_comments: false,
        },
    );
    assert_error!(
        result,
        ErrorKind::UnexpectedEof,
        "expression",
        at: (1, 8)
    );
}
