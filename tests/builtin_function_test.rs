mod common;

use veltrano::Config;

#[test]
fn test_println_builtin_function_type_checking() {
    let code = r#"
fun main() {
    println("Hello, world!")
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // Parse and type check the code
    let result = common::parse_and_type_check(code, config);

    // Should succeed because println is a built-in function
    assert!(
        result.is_ok(),
        "println should be recognized as a built-in function"
    );
}

#[test]
fn test_clone_builtin_method_type_checking() {
    let code = r#"
fun main() {
    val x = 42
    val y = x.ref().clone()
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // Parse and type check the code
    let result = common::parse_and_type_check(code, config);

    // Should succeed because .clone() is supported on Int
    assert!(
        result.is_ok(),
        "clone should be recognized as a built-in method on Int: {:?}",
        result
    );
}

#[test]
fn test_tostring_builtin_method_type_checking() {
    let code = r#"
fun main() {
    val x = 42
    val s = x.ref().toString()
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // Parse and type check the code
    let result = common::parse_and_type_check(code, config);

    // Should succeed because .toString() is supported on Int
    assert!(
        result.is_ok(),
        "toString should be recognized as a built-in method on Int: {:?}",
        result
    );
}
