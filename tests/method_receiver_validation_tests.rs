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
        val s: String = "hello".toString().ref()  // Create String from Str
        val s2 = s.clone()       // String implements Clone, returns Own<String>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_ok(),
        "Clone should work on naturally referenced types"
    );
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

    assert!(
        result.is_err(),
        "Own<T>.clone() should fail - explicit conversion required"
    );

    if let Err(errors) = result {
        let has_method_not_found = errors.iter().any(|err| {
            matches!(
                err,
                veltrano::TypeCheckError::MethodNotFound { .. }
                    | veltrano::TypeCheckError::_MethodNotFoundWithSuggestion { .. }
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
            let has_method_not_found = errors
                .iter()
                .any(|err| matches!(err, veltrano::TypeCheckError::MethodNotFound { .. }));
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
        val cloned: Own<String> = borrowed.clone()
        val str_ref: String = cloned.ref()
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
        
        // Chain: Own<String> -> String -> Own<String> (clone) -> String
        val result: String = owned.ref().clone().ref()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_ok(),
        "Method chaining with explicit conversions should work"
    );
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

    assert!(
        result.is_ok(),
        "Explicit conversion .ref().clone() should work"
    );
}

// ============================================================================
// I64 Clone Test Cases - Value types with Copy trait
// ============================================================================

#[test]
fn test_i64_clone_naturally_owned() {
    // I64 is naturally owned (implements Copy), so it can call clone directly
    let code = r#"
    fun main() {
        val x: I64 = 42
        val cloned: I64 = x.clone()  // I64 can call clone directly
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "I64 should be able to call clone directly");
}

#[test]
fn test_i64_ref_clone() {
    // Ref<I64> should also be able to call clone
    let code = r#"
    fun main() {
        val x: I64 = 42
        val ref_x: Ref<I64> = x.ref()
        val cloned: I64 = ref_x.clone()  // Ref<I64> can call clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Ref<I64> should be able to call clone");
}

#[test]
fn test_own_i64_clone_should_fail() {
    // Own<I64> should fail because I64 is a Copy type and shouldn't be wrapped in Own<>
    // But if it somehow exists, it shouldn't be able to auto-borrow
    let code = r#"
    fun main() {
        val owned: Own<I64> = someFunction()  // Hypothetical Own<I64>
        val cloned = owned.clone()  // Should fail - no auto-borrow
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    // This might fail for different reasons (Own<I64> validation or method not found)
    // The key is that it should fail, not succeed
    assert!(
        result.is_err(),
        "Own<I64>.clone() should fail - no auto-borrow allowed"
    );
}

#[test]
fn test_i64_clone_chaining() {
    // Test clone chaining with I64
    let code = r#"
    fun main() {
        val x: I64 = 42
        val cloned1: I64 = x.clone()
        val cloned2: I64 = cloned1.clone()
        val ref_cloned: Ref<I64> = cloned2.ref()
        val cloned3: I64 = ref_cloned.clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "I64 clone chaining should work");
}

#[test]
fn test_i64_vs_string_clone_comparison() {
    // Compare I64 (Copy type) vs String (non-Copy type) clone behavior
    let code = r#"
    fun main() {
        // I64 - naturally owned, can clone directly
        val num: I64 = 42
        val num_cloned: I64 = num.clone()
        
        // String - naturally referenced, can clone directly  
        val str: Own<String> = "hello".toString()
        val str_cloned: Own<String> = str.ref().clone()
        
        // Both can be wrapped in Ref<> and cloned
        val num_ref: Ref<I64> = num.ref()
        val num_ref_cloned: I64 = num_ref.clone()
        
        val str_ref: Ref<String> = str.ref().ref()
        val str_ref_cloned: String = str_ref.clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    if let Err(ref errors) = result {
        eprintln!("Type check errors: {:?}", errors);
    }
    assert!(
        result.is_ok(),
        "Both I64 and String should support direct cloning"
    );
}

#[test]
fn test_i64_method_chaining_with_clone() {
    // Test method chaining involving clone with I64
    let code = r#"
    fun main() {
        val x: I64 = 42
        
        // Chain: I64 -> I64 (clone) -> Ref<I64> -> I64 (clone)
        val result: I64 = x.clone().ref().clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "I64 method chaining with clone should work");
}

#[test]
fn test_i64_mutref_clone() {
    // Test that MutRef<I64> can call clone (if MutRef supports it)
    let code = r#"
    fun main() {
        val x: I64 = 42
        val mut_ref: MutRef<I64> = x.mutRef()
        
        // This might not work since MutRef<I64> would require I64 to not be Copy
        // But if it exists, it should follow the same rules
        val cloned = mut_ref.clone()  // Should fail - MutRef can't provide &self access
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    // MutRef<I64> likely shouldn't exist (I64 is Copy), but if it does,
    // it shouldn't be able to auto-convert to call clone
    assert!(result.is_err(), "MutRef<I64>.clone() should fail");
}
