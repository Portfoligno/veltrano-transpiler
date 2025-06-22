//! Code formatting utilities.
//!
//! Handles parameter and argument formatting for functions and structs.

use super::{CodeGenerator, CodegenError};
use crate::ast::*;
use crate::ast::Argument;
use crate::comments::{Comment, CommentStyle};
use crate::error::{Span, VeltranoError};
use crate::rust_interop::camel_to_snake_case;

impl CodeGenerator {
    /// Generate comma-separated parameters for function declarations
    pub(super) fn generate_comma_separated_params(
        &mut self,
        params: &[Parameter],
        include_bump: bool,
    ) {
        let mut first = true;

        if include_bump {
            self.output.push_str("bump: &'a bumpalo::Bump");
            first = false;
        }

        for param in params {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            let snake_name = camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type.node);
            self.generate_inline_comment_as_block(&param.inline_comment);
        }
    }

    /// Generate multiline parameters for function declarations
    pub(super) fn generate_multiline_params(&mut self, params: &[Parameter], include_bump: bool) {
        self.output.push('\n');
        self.indent_level += 1;

        if include_bump {
            self.indent();
            self.output.push_str("bump: &'a bumpalo::Bump");
            if !params.is_empty() {
                self.output.push(',');
            }
            self.output.push('\n');
        }

        for (i, param) in params.iter().enumerate() {
            self.indent();
            let snake_name = camel_to_snake_case(&param.name);
            self.output.push_str(&snake_name);
            self.output.push_str(": ");
            self.generate_type(&param.param_type.node);

            // Add comma if not the last parameter
            if i < params.len() - 1 {
                self.output.push(',');
            }

            // Generate inline comment as line comment, not block comment
            self.generate_inline_comment(&param.inline_comment);
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.indent();
    }

    /// Generate comma-separated arguments for struct initialization
    pub(super) fn generate_comma_separated_args_for_struct_init(
        &mut self,
        args: &[Argument],
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        let mut first = true;
        for arg in args {
            if !first {
                self.output.push_str(", ");
            }
            first = false;
            match arg {
                Argument::Bare(_, _) => {
                    // We need the data class name for the error, but we don't have it here
                    // For now, use a generic message
                    return Err(CodegenError::InvalidDataClassSyntax {
                        constructor: "data class".to_string(),
                        reason: "Data class constructors don't support positional arguments. Use named arguments or .field shorthand syntax".to_string(),
                        location: call_span.start.clone(),
                    }.into());
                }
                Argument::Named(name, expr, comment) => {
                    self.output.push_str(&camel_to_snake_case(name));
                    self.output.push_str(": ");
                    self.generate_expression(expr)?;
                    let comment_to_use = comment.before.as_ref().or(comment.after.as_ref());
                    self.generate_inline_comment(&comment_to_use.cloned());
                }
                Argument::Shorthand(field_name, comment) => {
                    // Shorthand: generate field_name (variable matches field name)
                    self.output.push_str(&camel_to_snake_case(field_name));
                    let comment_to_use = comment.before.as_ref().or(comment.after.as_ref());
                    self.generate_inline_comment(&comment_to_use.cloned());
                }
                Argument::StandaloneComment(_, _) => {
                    // For single-line struct initialization, ignore standalone comments
                    first = true; // Don't add comma before next real argument
                }
            }
        }
        Ok(())
    }

    /// Generate comma-separated arguments for function calls with multiline support
    pub(super) fn generate_comma_separated_args_for_function_call_with_multiline(
        &mut self,
        args: &[Argument],
        is_multiline: bool,
        call_span: Span,
    ) -> Result<(), VeltranoError> {
        if is_multiline && !args.is_empty() {
            // Generate multiline format
            self.output.push('\n');
            for (i, arg) in args.iter().enumerate() {
                self.indent_level += 1;
                self.indent();

                match arg {
                    Argument::Bare(expr, comment) => {
                        if let Some(ref before_comment) = comment.before {
                            self.generate_inline_comment(&Some(before_comment.clone()));
                            self.output.push(' ');
                        }
                        self.generate_expression(expr)?;
                        if i < args.len() - 1 {
                            self.output.push(',');
                        }
                        self.generate_inline_comment(&comment.after);
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        if let Some(ref before_comment) = comment.before {
                            self.generate_inline_comment(&Some(before_comment.clone()));
                            self.output.push(' ');
                        }
                        self.generate_expression(expr)?;
                        if i < args.len() - 1 {
                            self.output.push(',');
                        }
                        self.generate_inline_comment(&comment.after);
                    }
                    Argument::Shorthand(field_name, _) => {
                        return Err(CodegenError::InvalidShorthandUsage {
                            field_name: field_name.clone(),
                            context: "function calls".to_string(),
                            location: call_span.start.clone(),
                        }
                        .into());
                    }
                    Argument::StandaloneComment(content, whitespace) => {
                        // Generate standalone comment as its own line
                        if self.config.preserve_comments {
                            // Following the pattern from generate_comment for regular statements:
                            // The loop already called indent() to add base indentation.
                            // Now add any extra whitespace preserved by the lexer.
                            let comment =
                                Comment::from_tuple((content.clone(), whitespace.clone()));
                            self.output.push_str(&comment.whitespace);

                            match comment.style {
                                CommentStyle::Block => {
                                    self.output.push_str(&comment.content);
                                }
                                CommentStyle::Line => {
                                    // Check if it already has the prefix
                                    if comment.content.starts_with("//") {
                                        self.output.push_str(&comment.content);
                                    } else {
                                        self.output.push_str("//");
                                        self.output.push_str(&comment.content);
                                    }
                                }
                            }
                        }
                        // Note: No comma or expression - this is just a comment line
                    }
                }
                self.output.push('\n');
                self.indent_level -= 1;
            }
            self.indent();
        } else {
            // Generate single line format
            let mut first = true;
            for arg in args {
                if !first {
                    self.output.push(',');
                    // Check if current argument (that we're about to process) has a before comment
                    let current_has_before_comment = match arg {
                        Argument::Bare(_, comment) => comment.before.is_some(),
                        Argument::Named(_, _, comment) => comment.before.is_some(),
                        Argument::Shorthand(_, comment) => comment.before.is_some(),
                        Argument::StandaloneComment(_, _) => false,
                    };
                    if !current_has_before_comment {
                        self.output.push(' ');
                    }
                }
                first = false;
                match arg {
                    Argument::Bare(expr, comment) => {
                        if let Some(ref before_comment) = comment.before {
                            self.generate_inline_comment_as_block(&Some(before_comment.clone()));
                            self.output.push(' ');
                        }
                        self.generate_expression(expr)?;
                        if let Some(ref after_comment) = comment.after {
                            // Don't add extra space - the comment's whitespace already includes it
                            self.generate_inline_comment_as_block(&Some(after_comment.clone()));
                        }
                    }
                    // For function calls, named arguments are just treated as positional
                    Argument::Named(_, expr, comment) => {
                        if let Some(ref before_comment) = comment.before {
                            self.generate_inline_comment_as_block(&Some(before_comment.clone()));
                            self.output.push(' ');
                        }
                        self.generate_expression(expr)?;
                        if let Some(ref after_comment) = comment.after {
                            // Don't add extra space - the comment's whitespace already includes it
                            self.generate_inline_comment_as_block(&Some(after_comment.clone()));
                        }
                    }
                    Argument::Shorthand(field_name, _) => {
                        return Err(CodegenError::InvalidShorthandUsage {
                            field_name: field_name.clone(),
                            context: "function calls".to_string(),
                            location: call_span.start.clone(),
                        }
                        .into());
                    }
                    Argument::StandaloneComment(_, _) => {
                        // For single-line calls, standalone comments force multiline format
                        // This should not happen in practice as standalone comments should trigger multiline
                        // But handle it gracefully by ignoring the comment in single-line format
                        first = true; // Don't add comma before next real argument
                    }
                }
            }
        }
        Ok(())
    }
}
