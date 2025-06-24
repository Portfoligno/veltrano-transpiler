/// Tests for built-in import shadowing behavior
/// 
/// These tests verify that:
/// 1. Built-in imports work without user imports
/// 2. User imports completely shadow built-in imports
/// 3. No fallback to built-in when user import doesn't match
/// 4. Error messages are clear when methods aren't found

use veltrano::config::Config;

mod common;
use common::parse_and_type_check;

// ============================================================================
// Test 1: Built-in imports work without user imports
// ============================================================================

#[test]
fn test_builtin_clone_works_without_import() {
    // Clone should work out of the box as a built-in import
    let code = r#"
    fun main() {
        val x: I64 = 42
        val x_ref: Ref<I64> = x.ref()
        val cloned: I64 = x_ref.clone()  // Built-in Clone.clone
        
        val s: String = "hello".toString().ref()
        val s_cloned: Own<String> = s.clone()  // Built-in Clone.clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in clone() should work without explicit import");
}

#[test]
fn test_builtin_tostring_works_without_import() {
    // toString should work out of the box as a built-in import
    let code = r#"
    fun main() {
        val x: I64 = 42
        val s: Own<String> = x.ref().toString()  // Built-in ToString.toString
        
        val b: Bool = true
        val bs: Own<String> = b.ref().toString()  // Built-in ToString.toString
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in toString() should work without explicit import");
}

#[test]
fn test_builtin_length_works_without_import() {
    // length should work out of the box as a built-in import (aliased from len)
    let code = r#"
    fun main() {
        // String.length is a built-in import
        val s: Own<String> = "hello".toString()
        val len = s.ref().length()  // Built-in String.len aliased as length
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in length() should work without explicit import");
}

// ============================================================================
// Test 2: User imports completely shadow built-in imports
// ============================================================================

#[test]
fn test_user_import_shadows_builtin_clone() {
    // User import of Clone.clone should completely replace the built-in
    let code = r#"
    import MyClone.clone
    
    fun main() {
        val x: I64 = 42
        val x_ref: Ref<I64> = x.ref()
        // This should fail because MyClone.clone doesn't exist
        // It should NOT fall back to the built-in Clone.clone
        val cloned = x_ref.clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_err(), "User import should shadow built-in completely");
    
    if let Err(error) = result {
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::InvalidMethodCall),
            "Should fail when trying to use non-existent MyClone.clone"
        );
    }
}

#[test]
fn test_user_import_shadows_builtin_with_different_type() {
    // Test with a real type that has clone but isn't what we expect
    let code = r#"
    import String.clone as clone
    
    fun main() {
        val x: I64 = 42
        val x_ref: Ref<I64> = x.ref()
        // This should fail because String.clone doesn't match Ref<I64>
        // It should NOT fall back to the built-in Clone.clone
        val cloned = x_ref.clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_err(), "User import with wrong type should not fall back to built-in");
    
    if let Err(error) = result {
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::InvalidMethodCall),
            "Should have method not found error for type mismatch"
        );
    }
}

#[test]
fn test_user_import_shadows_builtin_length() {
    // User import of a custom length method should shadow all built-in length methods
    let code = r#"
    import MyType.customLength as length
    
    fun main() {
        val s: Own<String> = "hello".toString()
        // This should fail because MyType.customLength doesn't exist
        // It should NOT fall back to the built-in String.length
        val len = s.ref().length()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_err(), "User import should shadow all built-in length methods");
}

// ============================================================================
// Test 3: Correct user imports work and shadow built-ins
// ============================================================================

#[test]
fn test_correct_user_import_shadows_builtin() {
    // When user import is correct, it should work and shadow the built-in
    let code = r#"
    import String.len as length
    
    fun main() {
        val s: Own<String> = "hello".toString()
        // This uses the user import, not the built-in
        // (though they're functionally the same in this case)
        val len = s.ref().length()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Correct user import should work and shadow built-in");
}

#[test]
fn test_user_import_with_different_alias() {
    // User can import with a different alias, shadowing built-in
    let code = r#"
    import Clone.clone as duplicate
    
    fun main() {
        val x: I64 = 42
        val x_ref: Ref<I64> = x.ref()
        
        // clone() is no longer available (shadowed by different alias)
        // val bad = x_ref.clone()  // This would fail
        
        // But duplicate() works
        val dup: I64 = x_ref.duplicate()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "User import with different alias should work");
}

// ============================================================================
// Test 4: Multiple built-in imports work independently
// ============================================================================

#[test]
fn test_multiple_builtin_imports_independent() {
    // Different built-in imports should work independently
    let code = r#"
    fun main() {
        // All built-in imports available
        val x: I64 = 42
        val x_ref: Ref<I64> = x.ref()
        val cloned: I64 = x_ref.clone()  // Built-in Clone.clone
        
        val s1: Own<String> = x.ref().toString()  // Built-in ToString.toString
        
        val s2: Own<String> = "world".toString()
        val len = s2.ref().length()  // Built-in String.len as length
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Multiple built-in imports should work independently");
}

#[test]
fn test_shadowing_one_builtin_leaves_others() {
    // Shadowing one built-in import shouldn't affect others
    let code = r#"
    import MyType.myClone as clone
    
    fun main() {
        // clone is shadowed, but toString and length still work
        val x: I64 = 42
        val s: Own<String> = x.ref().toString()  // Built-in still works
        
        val s2: Own<String> = "world".toString()
        val len = s2.ref().length()  // Built-in still works
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    // This will fail because MyType.myClone doesn't exist,
    // but toString and length still work as built-ins
    assert!(result.is_ok(), "Other built-ins should remain available when one is shadowed");
}

// ============================================================================
// Test 5: Error messages are clear
// ============================================================================

#[test]
fn test_clear_error_when_shadowed_import_fails() {
    // Error message should be clear when shadowed import doesn't match
    let code = r#"
    import i64.abs as clone  // Shadow clone with something unrelated
    
    fun main() {
        val s: String = "hello".toString().ref()
        val bad = s.clone()  // Should fail - i64.abs doesn't work on String
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_err(), "Type mismatch should cause error");
    
    if let Err(error) = result {
        // Should get a clear "method not found" error, not ambiguity
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::InvalidMethodCall),
            "Should have clear method not found error"
        );
    }
}

#[test]
fn test_no_builtin_fallback_on_type_mismatch() {
    // When user import exists but types don't match, should not fall back to built-in
    let code = r#"
    import String.len as clone  // Shadow clone with len (wrong signature)
    
    fun main() {
        val x: I64 = 42
        val x_ref: Ref<I64> = x.ref()
        val bad = x_ref.clone()  // Should fail - String.len doesn't work on Ref<I64>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_err(), "Should not fall back to built-in on type mismatch");
}

// ============================================================================
// Test 6: Built-in aliasing works correctly
// ============================================================================

#[test]
fn test_builtin_length_alias_multiple_types() {
    // The built-in 'length' is an alias for multiple .len() methods
    let code = r#"
    fun main() {
        val s1: Own<String> = "hello".toString()
        val s1_len = s1.ref().length()  // String.len as length
        
        val s2: String = s1.ref()
        val s2_len = s2.length()  // Also works on String (not Own<String>)
        
        // Both work through the aliasing system
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config);
    assert!(result.is_ok(), "Built-in aliasing should work for multiple types");
}
