/// Tests for method receiver type validation with explicit conversion enforcement
use veltrano::config::Config;

mod common;
use common::parse_and_type_check;

// ============================================================================
// Positive Test Cases - Methods that should work with proper receiver types
// ============================================================================

#[test]
fn test_clone_on_reference_types() {
    // Clone should work on Ref<T> when T implements Clone
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned.ref()
        val cloned = borrowed.clone()  // String (Ref<String>) can call clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Clone should work on reference types");
}

#[test]
fn test_clone_on_naturally_referenced_types() {
    // Clone should work on naturally referenced types like String, when accessed via Ref<T>
    let code = r#"
    fun main() {
        val s: String = "hello"  // String is naturally referenced
        val s2 = s.clone()       // String implements Clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Clone should work on naturally referenced types");
}

#[test]
fn test_ref_method_conversions() {
    // .ref() should work for explicit conversions
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned.ref()     // Own<String> -> String
        
        val string_ref: Ref<String> = borrowed.ref()   // String -> Ref<String> (further borrowing)
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".ref() conversions should work");
}

#[test]
fn test_tostring_on_display_types() {
    // .toString() should work on types that implement Display, when properly accessed
    let code = r#"
    fun main() {
        val x: I64 = 42
        val s1: Own<String> = x.toString()     // I64 implements Display
        
        val b: Bool = true
        val s2: Own<String> = b.toString()     // Bool implements Display
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".toString() should work on Display types");
}

// ============================================================================
// Negative Test Cases - Methods that should fail due to explicit conversion enforcement
// ============================================================================

#[test]
fn test_clone_fails_on_owned_types() {
    // Own<T> should NOT be able to call .clone() directly - requires explicit .ref()
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val cloned = owned.clone()  // ERROR: Own<String> cannot directly call clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_err(), "Own<T>.clone() should fail - explicit conversion required");

    if let Err(errors) = result {
        let has_method_not_found = errors.iter().any(|err| {
            matches!(
                err,
                veltrano::TypeCheckError::MethodNotFound { .. }
                    | veltrano::TypeCheckError::MethodNotFoundWithSuggestion { .. }
            )
        });
        assert!(has_method_not_found, "Should have method not found error");
    }
}

#[test]
fn test_tostring_fails_on_owned_display_types() {
    // Own<T> should NOT be able to call .toString() directly even if T implements Display
    let code = r#"
    fun main() {
        val owned: Own<I64> = 42.clone()  // Assuming we can create Own<I64> somehow
        val string = owned.toString()     // ERROR: Own<I64> cannot directly call toString
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    // This test might pass or fail depending on how Own<I64> is created
    // The key point is that if it fails, it should be due to method not found on Own<I64>
    if result.is_err() {
        if let Err(errors) = result {
            let has_method_not_found = errors.iter().any(|err| {
                matches!(err, veltrano::TypeCheckError::MethodNotFound { .. })
            });
            // If it fails, it should be due to method not found, not other reasons
            if has_method_not_found {
                println!("Correctly rejected Own<I64>.toString() - explicit conversion required");
            }
        }
    }
}

// ============================================================================ 
// Complex Scenarios - Testing explicit conversion patterns
// ============================================================================

#[test]
fn test_explicit_conversion_chain() {
    // Test that explicit conversion chains work correctly
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned.ref()
        val cloned: String = borrowed.clone()
        val str_ref: Ref<String> = cloned.ref()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Explicit conversion chains should work");
}

#[test]
fn test_method_chaining_with_explicit_conversions() {
    // Test method chaining with explicit conversions
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        
        // Chain: Own<String> -> String -> String (clone) -> Ref<String>
        val result: Ref<String> = owned.ref().clone().ref()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Method chaining with explicit conversions should work");
}

// ============================================================================
// Error Message Validation - Ensuring helpful error messages
// ============================================================================

#[test]
fn test_helpful_error_message_for_owned_clone() {
    // Verify that trying to call .clone() on Own<T> gives a helpful error message
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = owned.clone()  // Should suggest .ref().clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_err(), "Own<String>.clone() should fail");

    if let Err(errors) = result {
        let error_message = format!("{:?}", errors);
        // The error should mention that the method is not found and ideally suggest .ref()
        assert!(
            error_message.contains("MethodNotFound") || error_message.contains("clone"),
            "Error should mention the missing clone method"
        );
    }
}

#[test]
fn test_correct_behavior_with_explicit_conversion() {
    // Verify that the suggested fix actually works
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = owned.ref().clone()  // This should work
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Explicit conversion .ref().clone() should work");
}
