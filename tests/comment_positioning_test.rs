mod common;
use common::{transpile, TestContext};
use veltrano::config::Config;

#[test]
fn test_comment_before_first_function_arg() {
    let input = r#"fun add(a: I64, b: I64): I64 {
    return a + b
}

fun main() {
    add(/* before first */ 30, 40)
}
"#;

    let expected = r#"fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
fn main() {
    add(/* before first */ 30, 40);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_comment_before_first_arg_multiline() {
    let input = r#"fun add(a: I64, b: I64): I64 {
    return a + b
}

fun main() {
    add(
        /* before first */ 30,
        40
    )
}
"#;

    let expected = r#"fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
fn main() {
    add(
        /* before first */ 30,
        40
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_comment_before_middle_arg() {
    let input = r#"fun process(a: I64, b: I64, c: I64): I64 {
    return a + b + c
}

fun main() {
    process(10, /* before middle */ 20, 30)
}
"#;

    let expected = r#"fn process(a: i64, b: i64, c: i64) -> i64 {
    return a + b + c;
}
fn main() {
    process(10, /* before middle */ 20, 30);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_mixed_before_after_comments() {
    let input = r#"fun calc(a: I64, b: I64): I64 {
    return a * b
}

fun main() {
    calc(/* before */ 10 /* after */, 20)
}
"#;

    let expected = r#"fn calc(a: i64, b: i64) -> i64 {
    return a * b;
}
fn main() {
    calc(/* before */ 10 /* after */, 20);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}
