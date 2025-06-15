use veltrano::comments::{Comment, CommentStyle};

#[test]
fn test_comment_from_tuple() {
    let tuple = ("// test comment".to_string(), "  ".to_string());
    let comment = Comment::from_tuple(tuple);
    assert_eq!(comment.style, CommentStyle::Line);
    assert_eq!(comment.content, "// test comment");
    assert_eq!(comment.whitespace, "  ");
}

#[test]
fn test_block_comment_detection() {
    let tuple = ("/* block comment */".to_string(), "".to_string());
    let comment = Comment::from_tuple(tuple);
    assert_eq!(comment.style, CommentStyle::Block);
    assert!(comment.is_block());
    assert!(!comment.is_line());
}

#[test]
fn test_raw_content() {
    let line_comment = Comment::new("// test".to_string(), "".to_string(), CommentStyle::Line);
    assert_eq!(line_comment.raw_content(), " test");

    let block_comment = Comment::new(
        "/* test */".to_string(),
        "".to_string(),
        CommentStyle::Block,
    );
    assert_eq!(block_comment.raw_content(), " test ");
}

#[test]
fn test_raw_content_without_delimiters() {
    // Test when content doesn't have delimiters
    let line_comment = Comment::new(
        "test comment".to_string(),
        "".to_string(),
        CommentStyle::Line,
    );
    assert_eq!(line_comment.raw_content(), "test comment");

    let block_comment = Comment::new(
        "test block".to_string(),
        "".to_string(),
        CommentStyle::Block,
    );
    assert_eq!(block_comment.raw_content(), "test block");
}

#[test]
fn test_to_block_style() {
    let line_comment = Comment::new(
        "// test comment".to_string(),
        "  ".to_string(),
        CommentStyle::Line,
    );
    let block = line_comment.to_block_style();
    assert_eq!(block.content, "/* test comment */");
    assert_eq!(block.style, CommentStyle::Block);
    assert_eq!(block.whitespace, "  ");

    // Test with comment without // prefix
    let line_comment2 = Comment::new(
        "plain comment".to_string(),
        "".to_string(),
        CommentStyle::Line,
    );
    let block2 = line_comment2.to_block_style();
    assert_eq!(block2.content, "/* plain comment */");

    // Test that block comments are unchanged
    let block_comment = Comment::new(
        "/* already block */".to_string(),
        "".to_string(),
        CommentStyle::Block,
    );
    let same_block = block_comment.to_block_style();
    assert_eq!(same_block.content, "/* already block */");
}

#[test]
fn test_format() {
    let comment = Comment::new("// test".to_string(), "".to_string(), CommentStyle::Line);
    assert_eq!(comment.format(false), "// test");
    assert_eq!(comment.format(true), "/* test */");

    let block_comment = Comment::new(
        "/* block */".to_string(),
        "".to_string(),
        CommentStyle::Block,
    );
    assert_eq!(block_comment.format(false), "/* block */");
    assert_eq!(block_comment.format(true), "/* block */"); // Already block style
}

#[test]
fn test_to_tuple_round_trip() {
    let original_tuple = ("// comment".to_string(), "    ".to_string());
    let comment = Comment::from_tuple(original_tuple.clone());
    let new_tuple = comment.to_tuple();
    assert_eq!(original_tuple, new_tuple);
}
