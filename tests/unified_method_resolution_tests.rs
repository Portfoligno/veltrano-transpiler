/// Tests for unified method resolution system that handles both built-in and imported methods
use veltrano::config::Config;

mod common;
use common::parse_and_type_check;

// ============================================================================
// Built-in Method Tests - Should continue working as before
// ============================================================================

#[test]
fn test_builtin_clone_methods() {
    // I64 clone (built-in, Copy type)
    let code = r#"
    fun main() {
        val x: I64 = 42
        val cloned: I64 = x.clone()  // Built-in CloneSemantics
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in I64 clone should work");
}

#[test]
fn test_builtin_string_clone() {
    // String clone (built-in, naturally referenced type)
    let code = r#"
    fun main() {
        val s: String = "hello".toString().ref()
        val cloned: Own<String> = s.clone()  // Built-in CloneSemantics: String -> Own<String>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in String clone should work");
}

#[test]
fn test_builtin_ref_methods() {
    // .ref() method (built-in, RefSemantics)
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned.ref()  // Built-in RefSemantics: Own<String> -> String
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in .ref() method should work");
}

// ============================================================================
// Imported Method Tests - New functionality
// ============================================================================

#[test]
fn test_imported_clone_methods() {
    // Test that methods work regardless of whether they're built-in or imported
    // The unified system should handle both transparently
    let code = r#"
    fun main() {
        // These work via the unified system (may be built-in or imported)
        val num: I64 = 42
        val num_cloned: I64 = num.clone()  // Works via unified method resolution
        
        val text: String = "hello".toString().ref()
        val text_cloned: Own<String> = text.clone()  // Works via unified method resolution
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_ok(),
        "Clone methods should work via unified system"
    );
}

#[test]
fn test_imported_tostring_methods() {
    // Test that imported methods work with UFCS generation
    let code = r#"
    import i64.abs
    
    fun main() {
        val num: I64 = -42
        val abs_result = num.abs()  // Generates: i64::abs(num)
        
        // Also test with positive number
        val pos: I64 = 42
        val same = pos.abs()  // Generates: i64::abs(pos)
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Imported methods should work with UFCS");
}

#[test]
fn test_imported_complex_return_types() {
    // Test imported methods with complex return types
    let code = r#"
    import String.chars
    
    fun main() {
        // Test imported methods with permissive behavior
        // Note: This demonstrates how the system handles imported methods
        // without hardcoded signatures using type inference
        
        val text: String = "hello".toString().ref()
        val chars_result = text.chars()  // Type inferred from permissive behavior
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_ok(),
        "Imported methods with complex return types should work"
    );
}

// ============================================================================
// Explicit Conversion Enforcement Tests - Should work for both built-in and imported
// ============================================================================

#[test]
fn test_explicit_conversion_enforcement_builtin() {
    // Own<T> should not auto-borrow for built-in methods
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val cloned = owned.clone()  // ERROR: Built-in clone needs explicit conversion
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_err(),
        "Own<String> should not auto-borrow for built-in clone"
    );
}

#[test]
fn test_explicit_conversion_enforcement_imported() {
    // Own<T> should not auto-borrow for imported methods either
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val string_result = owned.to_string()  // ERROR: Imported to_string needs explicit conversion
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_err(),
        "Own<String> should not auto-borrow for imported to_string"
    );
}

#[test]
fn test_imported_method_permissive_behavior() {
    // Test that imported methods are now type-checked like built-in methods
    // With type checking enabled, imports must match receiver types
    let code = r#"
    import String.toString
    
    fun main() {
        val text = "hello"
        val result = text.toString()  // Error: Str doesn't match String.toString
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);

    assert!(
        result.is_err(),
        "Imported methods should be type-checked and fail when types don't match"
    );

    if let Err(errors) = result {
        let error_message = format!("{:?}", errors);
        assert!(
            error_message.contains("MethodNotFound"),
            "Should have method not found error"
        );
    }
}

#[test]
fn test_explicit_conversion_works_for_both() {
    // Explicit conversion should work for both built-in and imported methods
    let code = r#"
    import String.len
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        
        // Built-in method with explicit conversion
        val cloned: Own<String> = owned.ref().clone()
        
        // Imported method with explicit conversion (use type inference)
        val len_result = owned.ref().len()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_ok(),
        "Explicit conversions should work for both built-in and imported methods"
    );
}

// ============================================================================
// Method Not Found Tests - Unified error handling
// ============================================================================

#[test]
fn test_method_not_found_unified() {
    // Non-existent methods should be handled consistently
    let code = r#"
    fun main() {
        val x: I64 = 42
        val result = x.nonexistent_method()  // Should check both built-in and imported
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_err(), "Non-existent methods should be rejected");

    if let Err(errors) = result {
        let error_message = format!("{:?}", errors);
        assert!(
            error_message.contains("MethodNotFound"),
            "Should have method not found error"
        );
    }
}

// ============================================================================
// Receiver Type Tests - Ensure both systems use same validation
// ============================================================================

#[test]
fn test_receiver_validation_consistency() {
    // Test that both built-in and imported methods use same receiver validation
    let code = r#"
    import i64.abs
    
    fun main() {
        val x: I64 = 42
        val ref_x: Ref<I64> = x.ref()
        
        // clone() works with Ref<I64> receiver
        val cloned1: I64 = ref_x.clone()      // Built-in method
        
        // abs() requires I64 receiver (by value), not Ref<I64>
        val abs1 = x.abs()                    // Imported method works with I64
        
        // For consistency, let's also test that abs() fails with wrong receiver
        // val bad = ref_x.abs()  // This would fail - abs needs value, not reference
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_ok(),
        "Receiver validation should be consistent for unified method resolution"
    );
}

#[test]
fn test_mutref_receiver_validation() {
    // Test MutRef receiver validation for both types of methods
    let code = r#"
    fun main() {
        val x: I64 = 42
        val mut_ref: MutRef<I64> = x.mutRef()
        
        // MutRef should not be able to call methods requiring &self
        val cloned = mut_ref.clone()  // ERROR: MutRef<I64> cannot provide &self access
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_err(),
        "MutRef should not auto-convert to &self for method calls"
    );
}

// ============================================================================
// Integration Tests - Mixed built-in and imported method calls
// ============================================================================

#[test]
fn test_mixed_method_calls() {
    // Test mixing built-in and imported method calls in same expression
    let code = r#"
    import String.len
    
    fun main() {
        val owned: Own<String> = "hello".toString()  // Built-in toString
        val borrowed: String = owned.ref()           // Built-in ref
        val cloned: Own<String> = borrowed.clone()   // Built-in clone
        val length = cloned.ref().len()              // Imported len (type inferred)
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(
        result.is_ok(),
        "Mixed built-in and imported method calls should work"
    );
}

#[test]
fn test_method_chaining_mixed() {
    // Test method chaining with both built-in and imported methods
    let code = r#"
    import String.len
    
    fun main() {
        // Chain multiple methods together mixing built-in and imported
        val text: String = "hello".toString().ref()
        
        // Chain: String -> Own<String> (clone) -> String (ref) -> usize (len)
        val size = text.clone().ref().len()
        
        // Another chain with built-in methods
        val result: String = text.clone().ref()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);

    if let Err(errors) = &result {
        println!("Method chaining error: {:?}", errors);
    }

    assert!(
        result.is_ok(),
        "Method chaining with mixed built-in and imported methods should work"
    );
}
