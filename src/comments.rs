//! Comment handling utilities for the Veltrano transpiler
//!
//! This module provides a unified representation for comments and utilities
//! for parsing, formatting, and manipulating comments throughout the transpiler.

/// Represents a comment with its content and metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// The comment content (without delimiters like // or /* */)
    pub content: String,
    /// Whitespace that precedes the comment
    pub whitespace: String,
    /// The style of the comment (line or block)
    pub style: CommentStyle,
}

/// Comment style (line vs block)
#[derive(Debug, Clone, PartialEq)]
pub enum CommentStyle {
    /// Line comment starting with //
    Line,
    /// Block comment surrounded by /* */
    Block,
}

impl Comment {
    /// Create a new comment
    pub fn new(content: String, whitespace: String, style: CommentStyle) -> Self {
        Comment {
            content,
            whitespace,
            style,
        }
    }

    /// Create from parser representation (content, whitespace) tuple
    pub fn from_tuple(tuple: (String, String)) -> Self {
        let (content, whitespace) = tuple;
        // Detect style from content
        let style = if content.starts_with("/*") && content.ends_with("*/") {
            CommentStyle::Block
        } else {
            CommentStyle::Line
        };
        Comment {
            content,
            whitespace,
            style,
        }
    }

    /// Convert to parser representation
    pub fn to_tuple(&self) -> (String, String) {
        (self.content.clone(), self.whitespace.clone())
    }

    /// Convert line comment to block style
    pub fn to_block_style(&self) -> Self {
        match self.style {
            CommentStyle::Block => self.clone(),
            CommentStyle::Line => {
                let content = if self.content.starts_with("//") {
                    format!("/* {} */", &self.content[2..].trim())
                } else {
                    format!("/* {} */", self.content.trim())
                };
                Comment {
                    content,
                    whitespace: self.whitespace.clone(),
                    style: CommentStyle::Block,
                }
            }
        }
    }

    /// Get raw content without delimiters
    pub fn raw_content(&self) -> &str {
        match self.style {
            CommentStyle::Line => {
                if self.content.starts_with("//") {
                    &self.content[2..]
                } else {
                    &self.content
                }
            }
            CommentStyle::Block => {
                let content = &self.content;
                if content.starts_with("/*") && content.ends_with("*/") {
                    &content[2..content.len() - 2]
                } else {
                    content
                }
            }
        }
    }

    /// Format for output with optional style conversion
    pub fn format(&self, force_block: bool) -> String {
        if force_block && self.style == CommentStyle::Line {
            self.to_block_style().content
        } else {
            self.content.clone()
        }
    }

    /// Check if this is a block comment
    pub fn is_block(&self) -> bool {
        matches!(self.style, CommentStyle::Block)
    }

    /// Check if this is a line comment
    pub fn is_line(&self) -> bool {
        matches!(self.style, CommentStyle::Line)
    }
}

/// Extension trait for AST nodes with comments
pub trait HasComment {
    /// Get the comment field
    fn comment(&self) -> Option<&(String, String)>;
    
    /// Get mutable reference to comment field
    fn comment_mut(&mut self) -> &mut Option<(String, String)>;

    /// Get comment as Comment struct
    fn get_comment(&self) -> Option<Comment> {
        self.comment().map(|c| Comment::from_tuple(c.clone()))
    }

    /// Set comment from Comment struct
    fn set_comment(&mut self, comment: Option<Comment>) {
        *self.comment_mut() = comment.map(|c| c.to_tuple());
    }

    /// Check if node has a comment
    fn has_comment(&self) -> bool {
        self.comment().is_some()
    }
}
