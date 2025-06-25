/// Tests that import errors are reported at the import statement location,
/// not at the use site.
use veltrano::config::Config;

mod common;
use common::{error_assertions::assert_error_location, parse_and_get_all_type_errors};

#[test]
fn test_invalid_type_import_error_at_import() {
    // MyFakeType doesn't exist as a trait or type, error should be at line 2 (import statement)
    let code = r#"
import MyFakeType.fakeMethod

fun main() {
    val result = fakeMethod()  // Line 5: Use site
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // Error should be at the import statement (line 2), not the use site (line 5)
    let result = parse_and_get_all_type_errors(code, config);

    match result {
        Ok(_) => panic!("Expected type check error for invalid import"),
        Err(errors) => {
            println!("Got {} errors", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("Error {}: {:?}", i, error);
                println!("Error {} message: {}", i, error.message);
                if let Some(span) = &error.context.span {
                    println!(
                        "Error {} location: {}:{}",
                        i, span.start.line, span.start.column
                    );
                }
            }

            // Find the import error
            let import_error = errors
                .iter()
                .find(|e| {
                    e.message.contains("Invalid import") && e.message.contains("has no method")
                })
                .expect("Should have an import error");

            assert_error_location(import_error, 2, 1);
        }
    }
}

#[test]
fn test_invalid_method_import_error_at_import() {
    // Vec exists but doesn't have a fakeMethod
    let code = r#"
import Vec.fakeMethod

fun main() {
    val result = fakeMethod()  // Line 5: Use site
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // Error should be at the import statement (line 2), not the use site (line 5)
    let result = parse_and_get_all_type_errors(code, config);

    match result {
        Ok(_) => panic!("Expected type check error for invalid import"),
        Err(errors) => {
            // Find the import error
            let import_error = errors
                .iter()
                .find(|e| {
                    e.message.contains("Invalid import") && e.message.contains("has no method")
                })
                .expect("Should have an import error");

            assert_error_location(import_error, 2, 1);
        }
    }
}

#[test]
fn test_valid_import_but_wrong_usage_error_at_use_site() {
    // Valid import, but used incorrectly (Vec.new requires type parameter)
    let code = r#"
import Vec.new

fun main() {
    val x: I64 = 42
    val result = x.new()  // Line 6: Wrong usage - new() is not a method on I64
}
"#;

    let config = Config {
        preserve_comments: false,
    };

    // Error should be at the use site (line 6) since the import is valid
    let result = parse_and_get_all_type_errors(code, config);

    match result {
        Ok(_) => panic!("Expected type check error for wrong usage"),
        Err(errors) => {
            // Should only have a use-site error (no import error since Vec.new is valid)
            assert_eq!(errors.len(), 1, "Should only have one error at use site");
            assert_error_location(&errors[0], 6, 18);
        }
    }
}
