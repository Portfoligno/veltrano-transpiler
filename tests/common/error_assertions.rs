//! Error assertion utilities for more precise error testing

use super::{parse_and_type_check, parse_veltrano_code};
use veltrano::config::Config;
use veltrano::error::{ErrorKind, VeltranoError};

/// Assert that an error has a specific kind
pub fn assert_error_kind(error: &VeltranoError, expected_kind: ErrorKind) {
    if error.kind != expected_kind {
        panic!(
            "Expected error kind {:?}, but got {:?}",
            expected_kind, error.kind
        );
    }
}

/// Assert that an error message contains a substring
pub fn assert_error_contains(error: &VeltranoError, substring: &str) {
    if !error.message.contains(substring) {
        panic!(
            "Expected error message to contain '{}', but got: '{}'",
            substring, error.message
        );
    }
}

/// Assert that an error has a specific location
pub fn assert_error_location(error: &VeltranoError, line: usize, column: usize) {
    if let Some(span) = &error.context.span {
        if span.start.line != line || span.start.column != column {
            panic!(
                "Expected error at {}:{}, but got {}:{}",
                line, column, span.start.line, span.start.column
            );
        }
    } else {
        panic!(
            "Expected error to have location {}:{}, but it has no span",
            line, column
        );
    }
}

/// Assert that an error spans specific locations
pub fn assert_error_span(error: &VeltranoError, start: (usize, usize), end: (usize, usize)) {
    if let Some(span) = &error.context.span {
        if span.start.line != start.0 || span.start.column != start.1 {
            panic!(
                "Expected error span to start at {}:{}, but got {}:{}",
                start.0, start.1, span.start.line, span.start.column
            );
        }
        if span.end.line != end.0 || span.end.column != end.1 {
            panic!(
                "Expected error span to end at {}:{}, but got {}:{}",
                end.0, end.1, span.end.line, span.end.column
            );
        }
    } else {
        panic!("Expected error to have span, but it has none");
    }
}

/// Assert that parsing fails with specific error properties
pub fn assert_parse_fails(code: &str, expected_kind: ErrorKind, message_contains: &str) {
    let config = Config {
        preserve_comments: false,
    };

    match parse_veltrano_code(code, config) {
        Ok(_) => panic!("Expected parsing to fail, but it succeeded"),
        Err(error) => {
            assert_error_kind(&error, expected_kind);
            assert_error_contains(&error, message_contains);
        }
    }
}

/// Assert that parsing fails at a specific location
pub fn assert_parse_fails_at(code: &str, expected_kind: ErrorKind, line: usize, column: usize) {
    let config = Config {
        preserve_comments: false,
    };

    match parse_veltrano_code(code, config) {
        Ok(_) => panic!("Expected parsing to fail, but it succeeded"),
        Err(error) => {
            assert_error_kind(&error, expected_kind);
            assert_error_location(&error, line, column);
        }
    }
}

/// Assert that type checking fails with specific error properties
pub fn assert_type_check_fails(code: &str, expected_kind: ErrorKind, message_contains: &str) {
    let config = Config {
        preserve_comments: false,
    };

    match parse_and_type_check(code, config) {
        Ok(_) => panic!("Expected type checking to fail, but it succeeded"),
        Err(error) => {
            assert_error_kind(&error, expected_kind);
            assert_error_contains(&error, message_contains);
        }
    }
}

/// Fluent assertion builder for errors
pub struct ErrorAssertion<'a> {
    error: &'a VeltranoError,
    failed: bool,
    failures: Vec<String>,
}

impl<'a> ErrorAssertion<'a> {
    /// Start a fluent assertion chain
    pub fn assert_that(error: &'a VeltranoError) -> Self {
        ErrorAssertion {
            error,
            failed: false,
            failures: Vec::new(),
        }
    }

    /// Assert error has specific kind
    pub fn has_kind(mut self, expected: ErrorKind) -> Self {
        if self.error.kind != expected {
            self.failed = true;
            self.failures.push(format!(
                "Expected error kind {:?}, but got {:?}",
                expected, self.error.kind
            ));
        }
        self
    }

    /// Assert error message contains substring
    pub fn has_message_containing(mut self, substring: &str) -> Self {
        if !self.error.message.contains(substring) {
            self.failed = true;
            self.failures.push(format!(
                "Expected message to contain '{}', but got: '{}'",
                substring, self.error.message
            ));
        }
        self
    }

    /// Assert error has exact message
    pub fn has_message(mut self, expected: &str) -> Self {
        if self.error.message != expected {
            self.failed = true;
            self.failures.push(format!(
                "Expected message '{}', but got: '{}'",
                expected, self.error.message
            ));
        }
        self
    }

    /// Assert error is at specific location
    pub fn at_location(mut self, line: usize, column: usize) -> Self {
        if let Some(span) = &self.error.context.span {
            if span.start.line != line || span.start.column != column {
                self.failed = true;
                self.failures.push(format!(
                    "Expected location {}:{}, but got {}:{}",
                    line, column, span.start.line, span.start.column
                ));
            }
        } else {
            self.failed = true;
            self.failures.push(format!(
                "Expected location {}:{}, but error has no span",
                line, column
            ));
        }
        self
    }

    /// Assert error is at specific line
    pub fn at_line(mut self, line: usize) -> Self {
        if let Some(span) = &self.error.context.span {
            if span.start.line != line {
                self.failed = true;
                self.failures.push(format!(
                    "Expected line {}, but got {}",
                    line, span.start.line
                ));
            }
        } else {
            self.failed = true;
            self.failures
                .push(format!("Expected line {}, but error has no span", line));
        }
        self
    }

    /// Assert error is at specific column
    pub fn at_column(mut self, column: usize) -> Self {
        if let Some(span) = &self.error.context.span {
            if span.start.column != column {
                self.failed = true;
                self.failures.push(format!(
                    "Expected column {}, but got {}",
                    column, span.start.column
                ));
            }
        } else {
            self.failed = true;
            self.failures
                .push(format!("Expected column {}, but error has no span", column));
        }
        self
    }

    /// Assert error has help text
    pub fn has_help_containing(mut self, substring: &str) -> Self {
        if let Some(help) = &self.error.context.help {
            if !help.contains(substring) {
                self.failed = true;
                self.failures.push(format!(
                    "Expected help to contain '{}', but got: '{}'",
                    substring, help
                ));
            }
        } else {
            self.failed = true;
            self.failures
                .push("Expected help text, but error has none".to_string());
        }
        self
    }

    /// Assert error has note text
    pub fn has_note_containing(mut self, substring: &str) -> Self {
        if let Some(note) = &self.error.context.note {
            if !note.contains(substring) {
                self.failed = true;
                self.failures.push(format!(
                    "Expected note to contain '{}', but got: '{}'",
                    substring, note
                ));
            }
        } else {
            self.failed = true;
            self.failures
                .push("Expected note text, but error has none".to_string());
        }
        self
    }

    /// Check if the error is a parse error
    pub fn is_parse_error(mut self) -> Self {
        match self.error.kind {
            ErrorKind::ParseError
            | ErrorKind::UnexpectedToken
            | ErrorKind::UnexpectedEof
            | ErrorKind::SyntaxError => {}
            _ => {
                self.failed = true;
                self.failures.push(format!(
                    "Expected parse error, but got {:?}",
                    self.error.kind
                ));
            }
        }
        self
    }

    /// Check if the error is a type error
    pub fn is_type_error(mut self) -> Self {
        match self.error.kind {
            ErrorKind::TypeError
            | ErrorKind::TypeMismatch
            | ErrorKind::InvalidMethodCall
            | ErrorKind::UndefinedVariable
            | ErrorKind::UndefinedFunction => {}
            _ => {
                self.failed = true;
                self.failures.push(format!(
                    "Expected type error, but got {:?}",
                    self.error.kind
                ));
            }
        }
        self
    }
}

impl<'a> Drop for ErrorAssertion<'a> {
    fn drop(&mut self) {
        if self.failed && !std::thread::panicking() {
            panic!("Error assertion failed:\n{}", self.failures.join("\n"));
        }
    }
}

/// Macro for concise error assertions
#[macro_export]
macro_rules! assert_error {
    // Simple kind check
    ($result:expr, $kind:expr) => {
        match $result {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(ref e) => $crate::common::error_assertions::assert_error_kind(e, $kind),
        }
    };

    // Kind with message pattern
    ($result:expr, $kind:expr, $msg:expr) => {
        match $result {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(ref e) => {
                $crate::common::error_assertions::assert_error_kind(e, $kind);
                $crate::common::error_assertions::assert_error_contains(e, $msg);
            }
        }
    };

    // Kind with message and location
    ($result:expr, $kind:expr, $msg:expr, at: ($line:expr, $col:expr)) => {
        match $result {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(ref e) => {
                $crate::common::error_assertions::assert_error_kind(e, $kind);
                $crate::common::error_assertions::assert_error_contains(e, $msg);
                $crate::common::error_assertions::assert_error_location(e, $line, $col);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_parse_fails() {
        assert_parse_fails("val x =", ErrorKind::UnexpectedEof, "expression");
    }

    #[test]
    fn test_fluent_assertions() {
        let code = "val x: String = 42";
        match parse_and_type_check(
            code,
            Config {
                preserve_comments: false,
            },
        ) {
            Ok(_) => panic!("Expected error"),
            Err(error) => {
                ErrorAssertion::assert_that(&error)
                    .is_type_error()
                    .has_message_containing("mismatch")
                    .at_line(1);
            }
        }
    }
}
