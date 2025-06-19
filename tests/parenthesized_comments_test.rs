mod common;
use common::{transpile, TestContext};
use veltrano::config::Config;

#[test]
fn test_comment_after_opening_paren() {
    let input = r#"fun main() {
    val x = (// comment
        10 + 20
    )
}"#;

    let expected = r#"fn main() {
    let x = (// comment
        10 + 20
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_block_comment_after_opening_paren() {
    let input = r#"fun main() {
    val x = (/* comment */
        10 + 20
    )
}"#;

    let expected = r#"fn main() {
    let x = (/* comment */
        10 + 20
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_comment_before_closing_paren() {
    let input = r#"fun main() {
    val x = (
        10 + 20
        // comment before closing
    )
}"#;

    let expected = r#"fn main() {
    let x = (
        10 + 20
        // comment before closing
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_multiple_comments_in_parens() {
    let input = r#"fun main() {
    val x = (
        // first comment
        10 + 
        /* middle */ 
        20
        // last comment
    )
}"#;

    let expected = r#"fn main() {
    let x = (// first comment
        10 + /* middle */ 20
        // last comment
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_nested_parens_with_comments() {
    let input = r#"fun main() {
    val x = (// outer comment
        10 + (// inner comment
            20 * 30
        )
    )
}"#;

    let expected = r#"fn main() {
    let x = (// outer comment
        10 + (// inner comment
            20 * 30
        )
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_inline_block_comment_in_parens() {
    let input = r#"fun main() {
    val x = (/* before */ 10 + 20 /* after */)
}"#;

    let expected = r#"fn main() {
    let x = (/* before */ 10 + 20 /* after */);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}
