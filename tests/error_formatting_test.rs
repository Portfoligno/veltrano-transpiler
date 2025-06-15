//! Tests for error formatting features

use veltrano::error::{ErrorFormatter, ErrorKind, SourceLocation, Span, VeltranoError};

#[test]
fn test_error_formatter_basic() {
    let source = "fun test(x) { x }";
    let error = VeltranoError::new(ErrorKind::SyntaxError, "Missing type annotation")
        .with_span(Span::single(SourceLocation::new(1, 11)));

    let formatter = ErrorFormatter::new(&error, source)
        .with_filename("test.vl")
        .with_color(false); // Disable color for testing

    let output = formatter.format();

    assert!(output.contains("test.vl:1:11:"));
    assert!(output.contains("syntax error: Missing type annotation"));
    assert!(output.contains("fun test(x) { x }"));
    assert!(output.contains("^")); // Error pointer
}

#[test]
fn test_error_formatter_with_context() {
    let source = "fun test(x) { x }";
    let error = VeltranoError::new(ErrorKind::SyntaxError, "Missing type annotation")
        .with_span(Span::single(SourceLocation::new(1, 11)))
        .with_note("Function parameters must have explicit types")
        .with_help("Try: x: i32");

    let formatter = ErrorFormatter::new(&error, source).with_color(false);

    let output = formatter.format();

    assert!(output.contains("note: Function parameters must have explicit types"));
    assert!(output.contains("help: Try: x: i32"));
}

#[test]
fn test_error_formatter_multiline_span() {
    let source = "fun test(\n    x\n) { x }";
    let error = VeltranoError::new(ErrorKind::SyntaxError, "Missing type annotation").with_span(
        Span::new(SourceLocation::new(2, 5), SourceLocation::new(2, 6)),
    );

    let formatter = ErrorFormatter::new(&error, source).with_color(false);

    let output = formatter.format();

    // Should show the line with the error
    assert!(output.contains("2 |     x"));
    assert!(output.contains("^")); // Points to 'x'
}

#[test]
fn test_error_collection_display() {
    use veltrano::error::ErrorCollection;

    let mut collection = ErrorCollection::new();

    collection.add_error(
        VeltranoError::new(ErrorKind::SyntaxError, "First error")
            .with_span(Span::single(SourceLocation::new(1, 5))),
    );

    collection.add_error(
        VeltranoError::new(ErrorKind::TypeError, "Second error")
            .with_span(Span::single(SourceLocation::new(3, 10))),
    );

    let output = collection.to_string();

    assert!(output.contains("error: 1:5: syntax error: First error"));
    assert!(output.contains("error: 3:10: type error: Second error"));
    assert!(output.contains("2 error(s), 0 warning(s)"));
}
