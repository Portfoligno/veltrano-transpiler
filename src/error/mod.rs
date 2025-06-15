//! Comprehensive error type hierarchy for the Veltrano transpiler
//!
//! This module provides a unified error handling system with rich context,
//! source location tracking, and user-friendly error messages.

mod conversions;

pub use conversions::IntoVeltranoError;

use colored::*;
use std::fmt;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            file: None,
            line,
            column,
        }
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file {
            Some(file) => write!(f, "{}:{}:{}", file, self.line, self.column),
            None => write!(f, "{}:{}", self.line, self.column),
        }
    }
}

/// Span information for multi-character error ranges
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl Span {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub fn single(location: SourceLocation) -> Self {
        Self {
            start: location.clone(),
            end: location,
        }
    }

    /// Get the start line number
    pub fn start_line(&self) -> usize {
        self.start.line
    }

    /// Get the start column number
    pub fn start_column(&self) -> usize {
        self.start.column
    }

    /// Get the end line number
    pub fn end_line(&self) -> usize {
        self.end.line
    }

    /// Get the end column number
    pub fn end_column(&self) -> usize {
        self.end.column
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// Error context providing additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub span: Option<Span>,
    pub note: Option<String>,
    pub help: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            span: None,
            note: None,
            help: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main error type for the Veltrano transpiler
#[derive(Debug, Clone)]
pub struct VeltranoError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: ErrorContext,
}

impl VeltranoError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.context.span = Some(span);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.context.note = Some(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.context.help = Some(help.into());
        self
    }
}

/// Categories of errors that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    // Lexer errors
    InvalidToken,
    UnterminatedString,
    InvalidNumber,
    InvalidCharacter,

    // Parser errors
    SyntaxError,
    UnexpectedToken,
    UnexpectedEof,
    InvalidExpression,
    InvalidStatement,

    // Type checker errors
    TypeError,
    UndefinedVariable,
    UndefinedFunction,
    UndefinedType,
    TypeMismatch,
    InvalidMethodCall,
    AmbiguousType,

    // Code generation errors
    CodegenError,
    UnsupportedFeature,
    InternalError,

    // Rust interop errors
    InteropError,
    CrateNotFound,
    ParseError,

    // IO errors
    IoError,
    FileNotFound,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::InvalidToken => "invalid token",
            ErrorKind::UnterminatedString => "unterminated string",
            ErrorKind::InvalidNumber => "invalid number",
            ErrorKind::InvalidCharacter => "invalid character",
            ErrorKind::SyntaxError => "syntax error",
            ErrorKind::UnexpectedToken => "unexpected token",
            ErrorKind::UnexpectedEof => "unexpected end of file",
            ErrorKind::InvalidExpression => "invalid expression",
            ErrorKind::InvalidStatement => "invalid statement",
            ErrorKind::TypeError => "type error",
            ErrorKind::UndefinedVariable => "undefined variable",
            ErrorKind::UndefinedFunction => "undefined function",
            ErrorKind::UndefinedType => "undefined type",
            ErrorKind::TypeMismatch => "type mismatch",
            ErrorKind::InvalidMethodCall => "invalid method call",
            ErrorKind::AmbiguousType => "ambiguous type",
            ErrorKind::CodegenError => "code generation error",
            ErrorKind::UnsupportedFeature => "unsupported feature",
            ErrorKind::InternalError => "internal error",
            ErrorKind::InteropError => "interop error",
            ErrorKind::CrateNotFound => "crate not found",
            ErrorKind::ParseError => "parse error",
            ErrorKind::IoError => "I/O error",
            ErrorKind::FileNotFound => "file not found",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for VeltranoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Error kind and location
        match &self.context.span {
            Some(span) => write!(f, "{}: {}: {}", span, self.kind, self.message)?,
            None => write!(f, "{}: {}", self.kind, self.message)?,
        }

        // Additional context
        if let Some(note) = &self.context.note {
            write!(f, "\nnote: {}", note)?;
        }

        if let Some(help) = &self.context.help {
            write!(f, "\nhelp: {}", help)?;
        }

        Ok(())
    }
}

/// Format error with source code snippet
pub struct ErrorFormatter<'a> {
    error: &'a VeltranoError,
    source: &'a str,
    filename: Option<&'a str>,
    use_color: bool,
}

impl<'a> ErrorFormatter<'a> {
    pub fn new(error: &'a VeltranoError, source: &'a str) -> Self {
        Self {
            error,
            source,
            filename: None,
            use_color: true,
        }
    }

    pub fn with_filename(mut self, filename: &'a str) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn format(&self) -> String {
        let mut output = String::new();

        // Error header
        if let Some(span) = &self.error.context.span {
            if let Some(filename) = self.filename {
                let location = format!("{}:{}:{}", filename, span.start.line, span.start.column);
                output.push_str(&if self.use_color {
                    location.bold().to_string()
                } else {
                    location
                });
                output.push_str(": ");
            } else {
                let location = format!("{}:{}", span.start.line, span.start.column);
                output.push_str(&if self.use_color {
                    location.bold().to_string()
                } else {
                    location
                });
                output.push_str(": ");
            }
        }

        let error_kind = self.error.kind.to_string();
        let error_label = if self.use_color {
            error_kind.red().bold().to_string()
        } else {
            error_kind
        };

        output.push_str(&format!("{}: {}\n", error_label, self.error.message));

        // Source code snippet
        if let Some(span) = &self.error.context.span {
            if let Some(snippet) = self.extract_snippet(span) {
                output.push_str(&snippet);
            }
        }

        // Additional context
        if let Some(note) = &self.error.context.note {
            let note_label = if self.use_color {
                "note".blue().bold()
            } else {
                "note".into()
            };
            output.push_str(&format!("\n{}: {}", note_label, note));
        }

        if let Some(help) = &self.error.context.help {
            let help_label = if self.use_color {
                "help".green().bold()
            } else {
                "help".into()
            };
            output.push_str(&format!("\n{}: {}", help_label, help));
        }

        output
    }

    fn extract_snippet(&self, span: &Span) -> Option<String> {
        let lines: Vec<&str> = self.source.lines().collect();

        // Ensure line numbers are valid (1-based)
        if span.start.line == 0 || span.start.line > lines.len() {
            return None;
        }

        let mut snippet = String::new();

        // Show the error line
        let line_idx = span.start.line - 1;
        let line = lines[line_idx];

        // Line number gutter width
        let gutter_width = span.start.line.to_string().len() + 2;

        // Add the source line with colored line number
        let line_num = span.start.line.to_string();
        let line_num_str = if self.use_color {
            line_num.blue().bold().to_string()
        } else {
            line_num
        };
        let separator = if self.use_color {
            "|".blue().to_string()
        } else {
            "|".to_string()
        };

        snippet.push_str(&format!(
            "{:>width$} {} {}\n",
            line_num_str,
            separator,
            line,
            width = gutter_width - 2
        ));

        // Add the error pointer
        let padding = " ".repeat(gutter_width);
        let separator_padding = if self.use_color {
            "|".blue().to_string()
        } else {
            "|".to_string()
        };
        let pointer_padding = " ".repeat(span.start.column.saturating_sub(1));
        let pointer_length = if span.start.line == span.end.line {
            span.end.column.saturating_sub(span.start.column).max(1)
        } else {
            1
        };
        let pointer = "^".repeat(pointer_length);
        let pointer_str = if self.use_color {
            pointer.red().bold().to_string()
        } else {
            pointer
        };

        snippet.push_str(&format!(
            "{} {} {}{}",
            padding, separator_padding, pointer_padding, pointer_str
        ));

        Some(snippet)
    }
}

impl std::error::Error for VeltranoError {}

/// Result type for Veltrano operations
pub type Result<T> = std::result::Result<T, VeltranoError>;

/// Collection of errors for reporting multiple issues
#[derive(Debug)]
pub struct ErrorCollection {
    errors: Vec<VeltranoError>,
    warnings: Vec<VeltranoError>,
}

impl ErrorCollection {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: VeltranoError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: VeltranoError) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn errors(&self) -> &[VeltranoError] {
        &self.errors
    }

    pub fn warnings(&self) -> &[VeltranoError] {
        &self.warnings
    }
}

impl Default for ErrorCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "error: {}", error)?;
        }

        for warning in &self.warnings {
            writeln!(f, "warning: {}", warning)?;
        }

        if !self.is_empty() {
            write!(
                f,
                "\n{} error(s), {} warning(s)",
                self.error_count(),
                self.warning_count()
            )?;
        }

        Ok(())
    }
}

/// Convert from other error types
impl From<std::io::Error> for VeltranoError {
    fn from(err: std::io::Error) -> Self {
        VeltranoError::new(ErrorKind::IoError, err.to_string())
    }
}
