//! Comment generation and placement.
//!
//! Preserves all comment styles with proper indentation.

use super::CodeGenerator;
use crate::ast::{CommentContext, CommentStmt};
use crate::comments::{Comment, CommentStyle};

/// Comment markers (for string output)
const DOUBLE_SLASH: &str = "//";
const SLASH_STAR: &str = "/*";
const STAR_SLASH: &str = "*/";

impl CodeGenerator {
    /// Generate a standalone comment statement
    pub(super) fn generate_comment(&mut self, comment: &CommentStmt) {
        match comment.context {
            CommentContext::OwnLine => {
                // Own-line comments get indentation
                self.indent();
            }
            CommentContext::EndOfLine => {
                // EndOfLine comments: remove the trailing newline from previous statement
                if self.output.ends_with('\n') {
                    self.output.pop();
                }
            }
        }

        // Always apply the preserved whitespace
        self.output.push_str(&comment.preceding_whitespace);

        if comment.is_block_comment {
            self.output.push_str(SLASH_STAR);
            self.output.push_str(&comment.content);
            self.output.push_str(STAR_SLASH);
        } else {
            self.output.push_str(DOUBLE_SLASH);
            self.output.push_str(&comment.content);
        }

        // Always add newline at the end
        self.output.push('\n');
    }

    /// Generate an inline comment (preserves line vs block style)
    pub(super) fn generate_inline_comment(&mut self, inline_comment: &Option<(String, String)>) {
        if let Some((content, whitespace)) = inline_comment {
            if self.config.preserve_comments {
                let comment = Comment::from_tuple((content.clone(), whitespace.clone()));
                self.output.push_str(&comment.whitespace);

                // Use Comment to determine style and format appropriately
                match comment.style {
                    CommentStyle::Block => {
                        // Block comment - output as-is
                        self.output.push_str(&comment.content);
                    }
                    CommentStyle::Line => {
                        // Line comment - check if it already has the prefix
                        if comment.content.starts_with(DOUBLE_SLASH) {
                            self.output.push_str(&comment.content);
                        } else {
                            self.output.push_str(DOUBLE_SLASH);
                            self.output.push_str(&comment.content);
                        }
                    }
                }
            }
        }
    }

    /// Generate an inline comment as block style (converts line comments to block)
    pub(super) fn generate_inline_comment_as_block(
        &mut self,
        inline_comment: &Option<(String, String)>,
    ) {
        if let Some((content, whitespace)) = inline_comment {
            if self.config.preserve_comments {
                let comment = Comment::from_tuple((content.clone(), whitespace.clone()));
                self.output.push_str(&comment.whitespace);

                // Use Comment's to_block_style method to convert if needed
                let block_comment = comment.to_block_style();
                self.output.push_str(&block_comment.content);
            }
        }
    }
}
