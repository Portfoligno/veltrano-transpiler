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

#[test]
fn test_single_line_comment_after_arg() {
    let input = r#"fun process(a: I64, b: I64) {
    println("{} {}", a, b)
}

fun main() {
    process(10 /* after first */, 20)
}"#;

    let expected = r#"fn process(a: i64, b: i64) {
    println!("{} {}", a, b);
}
fn main() {
    process(10 /* after first */, 20);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_line_comments_after_args_multiline() {
    let input = r#"fun compute(x: I64, y: I64, z: I64): I64 {
    return x + y + z
}

fun main() {
    val result = compute(
        10,  // first value
        20,  // second value
        30   // third value
    )
}"#;

    let expected = r#"fn compute(x: i64, y: i64, z: i64) -> i64 {
    return x + y + z;
}
fn main() {
    let result = compute(
        10,  // first value
        20,  // second value
        30   // third value
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_block_comments_complex_positioning() {
    let input = r#"fun process(a: I64, b: I64, c: I64, d: I64): I64 {
    return a + b + c + d
}

fun main() {
    process(/* a */ 1 /* after a */, /* b */ 2, 3 /* after c */, /* d */ 4)
}"#;

    let expected = r#"fn process(a: i64, b: i64, c: i64, d: i64) -> i64 {
    return a + b + c + d;
}
fn main() {
    process(/* a */ 1 /* after a */, /* b */ 2, 3 /* after c */, /* d */ 4);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_standalone_comment_lines_in_args() {
    let input = r#"fun calculate(a: I64, b: I64, c: I64): I64 {
    return a * b + c
}

fun main() {
    calculate(
        10,
        // This is a standalone comment
        20,
        // Another standalone comment
        // with multiple lines
        30
    )
}"#;

    let expected = r#"fn calculate(a: i64, b: i64, c: i64) -> i64 {
    return a * b + c;
}
fn main() {
    calculate(
        10,
        // This is a standalone comment
        20,
        // Another standalone comment
        // with multiple lines
        30
    );
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}

#[test]
fn test_nested_function_calls_with_comments() {
    let input = r#"fun add(a: I64, b: I64): I64 {
    return a + b
}

fun multiply(x: I64, y: I64): I64 {
    return x * y
}

fun main() {
    val result = add(
        /* first */ multiply(2 /* x */, 3 /* y */),
        /* second */ multiply(
            4,  // a
            5   // b
        )
    )
}"#;

    let expected = r#"fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
fn multiply(x: i64, y: i64) -> i64 {
    return x * y;
}
fn main() {
    let result = add(
        /* first */ multiply(2 /* x */, 3 /* y */),
        /* second */ multiply(
            4,  // a
            5   // b
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
fn test_empty_comment_positioning() {
    let input = r#"fun test(a: I64, b: I64) {
    println("{} {}", a, b)
}

fun main() {
    test(/**/ 10, 20 /**/)
}"#;

    let expected = r#"fn test(a: i64, b: i64) {
    println!("{} {}", a, b);
}
fn main() {
    test(/**/ 10, 20 /**/);
}"#;

    let ctx = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let result = transpile(input, &ctx).unwrap();
    assert_eq!(result.trim(), expected.trim());
}
