//! Snapshot tests for error messages

mod common;

use common::parse_and_type_check;
use common::snapshot_utils::assert_error_snapshot;
use veltrano::config::Config;
use veltrano::lexer::Lexer;
use veltrano::parser::Parser;

/// Helper to capture parse errors
fn get_parse_error(code: &str) -> String {
    let config = Config { preserve_comments: false };
    let mut lexer = Lexer::with_config(code.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(_) => panic!("Expected parse error but parsing succeeded"),
        Err(e) => format!("{}", e),
    }
}

/// Helper to capture type check errors
fn get_type_error(code: &str) -> String {
    let config = Config { preserve_comments: false };
    
    match parse_and_type_check(code, config) {
        Ok(_) => panic!("Expected type error but type checking succeeded"),
        Err(e) => format!("{}", e),
    }
}

#[test]
fn test_parse_error_missing_expression() {
    let error = get_parse_error("val x =");
    assert_error_snapshot("parse_error_missing_expression", &error);
}

// TODO: Known issue - parser currently accepts unclosed strings
// #[test]
// fn test_parse_error_unclosed_string() {
//     let error = get_parse_error(r#"val s = "hello"#);
//     assert_error_snapshot("parse_error_unclosed_string", &error);
// }

#[test]
fn test_parse_error_unexpected_token() {
    let error = get_parse_error("val x = 1 +");
    assert_error_snapshot("parse_error_unexpected_token", &error);
}

#[test]
fn test_parse_error_invalid_function() {
    let error = get_parse_error("fun () { }");
    assert_error_snapshot("parse_error_invalid_function", &error);
}

#[test]
fn test_parse_error_missing_closing_brace() {
    let error = get_parse_error("fun test() { val x = 1");
    assert_error_snapshot("parse_error_missing_closing_brace", &error);
}

#[test]
fn test_parse_error_invalid_pattern() {
    let error = get_parse_error("val (x, ) = tuple");
    assert_error_snapshot("parse_error_invalid_pattern", &error);
}

#[test]
fn test_type_error_undefined_variable() {
    let error = get_type_error("val x = undefined_var");
    assert_error_snapshot("type_error_undefined_variable", &error);
}

#[test]
fn test_type_error_type_mismatch() {
    let error = get_type_error(r#"
        val x: String = 42
    "#);
    assert_error_snapshot("type_error_type_mismatch", &error);
}

#[test]
fn test_type_error_invalid_own_usage() {
    let error = get_type_error("val x: Own<I64> = 42");
    assert_error_snapshot("type_error_invalid_own_i64", &error);
}

#[test]
fn test_type_error_double_own() {
    let error = get_type_error("val x: Own<Own<String>> = something");
    assert_error_snapshot("type_error_double_own", &error);
}

#[test]
fn test_type_error_own_mutref() {
    let error = get_type_error("val x: Own<MutRef<String>> = something");
    assert_error_snapshot("type_error_own_mutref", &error);
}

#[test]
fn test_type_error_invalid_method_call() {
    let error = get_type_error(r#"
        val x = 42
        val y = x.undefined_method()
    "#);
    assert_error_snapshot("type_error_invalid_method_call", &error);
}

#[test]
fn test_type_error_wrong_number_of_arguments() {
    let error = get_type_error(r#"
        fun add(a: I64, b: I64) -> I64 { a + b }
        val result = add(1)
    "#);
    assert_error_snapshot("type_error_wrong_number_of_arguments", &error);
}

#[test]
fn test_parse_error_double_minus() {
    let error = get_parse_error("val x = --5");
    assert_error_snapshot("parse_error_double_minus", &error);
}

#[test]
fn test_parse_error_invalid_match() {
    let error = get_parse_error(r#"
        match x {
            Some(value) =>
        }
    "#);
    assert_error_snapshot("parse_error_invalid_match", &error);
}
