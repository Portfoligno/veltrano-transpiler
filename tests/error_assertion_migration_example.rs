//! Example of migrating tests to use error assertion utilities

mod common;

use common::error_assertions::*;
use veltrano::error::ErrorKind;
use veltrano::config::Config;

// BEFORE: Old style error test
#[test]
fn test_old_style_error_check() {
    use common::parse_veltrano_code;
    
    let result = parse_veltrano_code("val x =", Config { preserve_comments: false });
    match result {
        Ok(_) => panic!("Expected parse error"),
        Err(error) => {
            // Manual assertions
            assert!(error.to_string().contains("Expected expression"));
            assert!(error.to_string().contains("1:8"));
        }
    }
}

// AFTER: Using error assertion utilities
#[test]
fn test_new_style_error_check() {
    // Much cleaner and more precise
    assert_parse_fails_at(
        "val x =",
        ErrorKind::UnexpectedEof,
        1, 8
    );
}

// BEFORE: Complex error checking with multiple properties
#[test]
fn test_old_style_complex_check() {
    use common::parse_and_type_check;
    
    let code = "val x: String = 42";
    match parse_and_type_check(code, Config { preserve_comments: false }) {
        Ok(_) => panic!("Expected type error"),
        Err(error) => {
            let error_str = error.to_string();
            assert!(error_str.contains("type mismatch"));
            assert!(error_str.contains("String"));
            assert!(error_str.contains("I64"));
            // Can't easily check exact location or error kind
        }
    }
}

// AFTER: Using fluent assertions
#[test]
fn test_new_style_complex_check() {
    use common::parse_and_type_check;
    
    let code = "val x: String = 42";
    match parse_and_type_check(code, Config { preserve_comments: false }) {
        Ok(_) => panic!("Expected type error"),
        Err(error) => {
            ErrorAssertion::assert_that(&error)
                .has_kind(ErrorKind::TypeMismatch)
                .has_message_containing("Type mismatch")
                .has_message_containing("String")
                .has_message_containing("I64")
                .at_line(1)
                .at_column(17); // Points to the 42
        }
    }
}

// BEFORE: Testing multiple error scenarios
#[test]
fn test_old_style_multiple_scenarios() {
    use common::{parse_veltrano_code, parse_and_type_check};
    
    // Test 1: Parse error
    let result1 = parse_veltrano_code("val =", Config { preserve_comments: false });
    assert!(result1.is_err());
    assert!(result1.unwrap_err().to_string().contains("Expected variable name"));
    
    // Test 2: Type error
    let result2 = parse_and_type_check("val x = unknown", Config { preserve_comments: false });
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Variable 'unknown' not found"));
    
    // Test 3: Another parse error
    let result3 = parse_veltrano_code("fun test(", Config { preserve_comments: false });
    assert!(result3.is_err());
    assert!(result3.unwrap_err().to_string().contains("Expected"));
}

// AFTER: Using specific assertion helpers
#[test]
fn test_new_style_multiple_scenarios() {
    // Test 1: Parse error
    assert_parse_fails(
        "val =",
        ErrorKind::SyntaxError,
        "Expected variable name"
    );
    
    // Test 2: Type error
    assert_type_check_fails(
        "val x = unknown",
        ErrorKind::UndefinedVariable,
        "Variable 'unknown' not found"
    );
    
    // Test 3: Another parse error
    assert_parse_fails(
        "fun test(",
        ErrorKind::UnexpectedEof,
        "Expected"
    );
}

// Example: Using the assert_error! macro for quick checks
#[test]
fn test_macro_examples() {
    use common::parse_veltrano_code;
    
    // Simple kind check
    let result = parse_veltrano_code("val x =", Config { preserve_comments: false });
    assert_error!(result, ErrorKind::UnexpectedEof);
    
    // With message check
    let result = parse_veltrano_code("val = 42", Config { preserve_comments: false });
    assert_error!(result, ErrorKind::SyntaxError, "variable name");
    
    // With location check
    let result = parse_veltrano_code("val x = 1 +", Config { preserve_comments: false });
    assert_error!(
        result,
        ErrorKind::UnexpectedEof,
        "expression",
        at: (1, 12)
    );
}