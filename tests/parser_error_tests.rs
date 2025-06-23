/// Parser error tests - Batch 14 Checkpoint 1
/// Tests that verify the parser correctly handles various error cases
/// Uses TODO markers for cases that should be fixed later
use veltrano::config::Config;
use veltrano::lexer::Lexer;
use veltrano::parser::Parser;

/// Helper that attempts to parse code and returns Ok(()) if it succeeds, Err(msg) if it fails
fn try_parse(code: &str) -> Result<(), String> {
    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(code.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse().map(|_| ()).map_err(|e| e.to_string())
}

// ============================================================================
// Lexer-level issues (TODOs)
// ============================================================================

#[test]
fn test_unterminated_strings() {
    // TODO: These should be caught by the lexer
    assert!(
        try_parse(r#"val x = "unterminated"#).is_ok(),
        "TODO: Unterminated strings are currently accepted by lexer"
    );

    assert!(
        try_parse(r#"val x = "unterminated \"#).is_ok(),
        "TODO: Unterminated strings with escapes are currently accepted"
    );
}

#[test]
fn test_unterminated_comments() {
    // TODO: Should be caught by the lexer
    assert!(
        try_parse("/* unterminated comment").is_ok(),
        "TODO: Unterminated block comments are currently accepted"
    );

    assert!(
        try_parse("/* nested /* comment */ unterminated").is_ok(),
        "TODO: Nested unterminated comments are currently accepted"
    );
}

#[test]
fn test_invalid_escape_sequences() {
    // TODO: Should validate escape sequences
    assert!(
        try_parse(r#"val x = "invalid \q escape""#).is_ok(),
        "TODO: Invalid escape sequences are currently accepted"
    );
}

// ============================================================================
// Number literal errors (correctly handled)
// ============================================================================

#[test]
fn test_invalid_number_literals() {
    // Hex without digits
    assert!(
        try_parse("val x = 0x").is_err(),
        "Hex prefix without digits correctly rejected"
    );

    // Multiple decimal points
    assert!(
        try_parse("val x = 1.2.3").is_err(),
        "Multiple decimal points correctly rejected"
    );

    // Trailing decimal point
    assert!(
        try_parse("val x = 123.").is_err(),
        "Trailing decimal point correctly rejected"
    );
}

// ============================================================================
// Statement and expression errors (correctly handled)
// ============================================================================

#[test]
fn test_statement_separation() {
    // Veltrano uses newlines, not semicolons
    assert!(
        try_parse("val x = 1 val y = 2").is_err(),
        "Multiple statements on one line correctly rejected"
    );

    assert!(
        try_parse("val x = 1; val y = 2").is_err(),
        "Semicolons don't separate statements"
    );

    assert!(
        try_parse("val x = 1\nval y = 2").is_ok(),
        "Newline-separated statements correctly accepted"
    );
}

#[test]
fn test_delimiter_matching() {
    assert!(
        try_parse("val x = (1 + 2").is_err(),
        "Unclosed parentheses correctly rejected"
    );

    assert!(
        try_parse("fun f() { val x = 1").is_err(),
        "Unclosed braces correctly rejected"
    );

    assert!(
        try_parse("val x = [1, 2, 3)").is_err(),
        "Mismatched brackets correctly rejected"
    );
}

#[test]
fn test_expression_errors() {
    assert!(
        try_parse("val x = ()").is_err(),
        "Empty parentheses correctly rejected"
    );

    assert!(
        try_parse("val x = 1 + + 2").is_err(),
        "Consecutive operators correctly rejected"
    );

    assert!(
        try_parse("val x = 1 +").is_err(),
        "Trailing operator correctly rejected"
    );

    assert!(
        try_parse("val x = .field").is_err(),
        "Field access without receiver correctly rejected"
    );
}

// ============================================================================
// Type annotation errors (correctly handled)
// ============================================================================

#[test]
fn test_type_annotations() {
    assert!(
        try_parse("val x: = 5").is_err(),
        "Missing type after colon correctly rejected"
    );

    assert!(
        try_parse("val x: Vec<").is_err(),
        "Unclosed generic type correctly rejected"
    );

    assert!(
        try_parse("val x: Vec<>").is_err(),
        "Empty generic type correctly rejected"
    );
}

// ============================================================================
// Function and class declarations (correctly handled)
// ============================================================================

#[test]
fn test_function_errors() {
    assert!(
        try_parse("fun").is_err(),
        "Incomplete function declaration correctly rejected"
    );

    assert!(
        try_parse("fun test()").is_err(),
        "Function without body correctly rejected"
    );

    assert!(
        try_parse("fun 123test() {}").is_err(),
        "Function name starting with digit correctly rejected"
    );
}

#[test]
fn test_data_class_errors() {
    assert!(
        try_parse("data class").is_err(),
        "Data class without name correctly rejected"
    );

    assert!(
        try_parse("data class Point").is_err(),
        "Data class without body correctly rejected"
    );

    assert!(
        try_parse("data class Point { x: }").is_err(),
        "Field without type correctly rejected"
    );
}

// ============================================================================
// Control flow errors (correctly handled)
// ============================================================================

#[test]
fn test_control_flow_errors() {
    assert!(
        try_parse("if { }").is_err(),
        "If without condition correctly rejected"
    );

    assert!(
        try_parse("if (true)").is_err(),
        "If without body correctly rejected"
    );

    assert!(
        try_parse("else { }").is_err(),
        "Else without if correctly rejected"
    );

    assert!(
        try_parse("match { }").is_err(),
        "Match without expression correctly rejected"
    );

    assert!(
        try_parse("match x { }").is_err(),
        "Match without arms correctly rejected"
    );
}

// ============================================================================
// Import errors (correctly handled)
// ============================================================================

#[test]
fn test_import_errors() {
    assert!(
        try_parse("import").is_err(),
        "Import without path correctly rejected"
    );

    assert!(
        try_parse("import .").is_err(),
        "Import with invalid path correctly rejected"
    );

    assert!(
        try_parse("import Vec.new as").is_err(),
        "Import with incomplete alias correctly rejected"
    );
}

// ============================================================================
// Identifier rules
// ============================================================================

#[test]
fn test_identifier_rules() {
    assert!(
        try_parse("val 123abc = 5").is_err(),
        "Identifiers starting with digits correctly rejected"
    );

    assert!(
        try_parse("val fun = 5").is_err(),
        "Keywords as identifiers correctly rejected"
    );

    assert!(
        try_parse("val class = 5").is_err(),
        "Reserved words as identifiers correctly rejected"
    );

    // TODO: Emoji identifiers
    assert!(
        try_parse("val 👋_hello = 5").is_ok(),
        "TODO: Emoji identifiers are currently accepted"
    );
}

// ============================================================================
// Edge cases that need decisions
// ============================================================================

#[test]
fn test_edge_cases() {
    // Chained comparisons - could be parse error or type error
    assert!(
        try_parse("val x = 1 < 2 < 3").is_ok(),
        "TODO: Chained comparisons accepted (will fail in type checking)"
    );

    // Null bytes in source
    assert!(
        try_parse("val x = 5\0val y = 6").is_err(),
        "Null bytes correctly treated as whitespace, causing statement error"
    );
}

// ============================================================================
// Error recovery testing
// ============================================================================

#[test]
fn test_error_recovery() {
    // Parser stops at first error
    let code = "fun broken() {\n    val x = \"unterminated\n    val y = (1 + 2\n}";
    assert!(
        try_parse(code).is_err(),
        "Parser reports error for malformed code"
    );

    // Multiple errors on same line
    let code = "val x = 1 + + 2 val y = 3";
    assert!(
        try_parse(code).is_err(),
        "Parser catches first error in line"
    );
}
