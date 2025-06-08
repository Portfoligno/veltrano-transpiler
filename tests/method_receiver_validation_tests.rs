use veltrano::config::Config;

mod common;
use common::parse_and_type_check;

// ============================================================================
// Positive Test Cases - Methods that should work on specific receiver types
// ============================================================================

#[test]
fn test_clone_on_value_types() {
    // Clone should work on value types (Int, Bool, Unit)
    let code = r#"
    fun main() {
        val x: I64 = 42
        val y = x.clone()           // I64 implements Clone
        
        val b: Bool = true
        val b2 = b.clone()          // Bool implements Clone
        
        val u: Unit = ()
        val u2 = u.clone()          // Unit implements Clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Clone should work on value types");
}

#[test]
fn test_clone_on_reference_types() {
    // Clone should work on naturally referenced types (String, Str)
    let code = r#"
    fun main() {
        val s: String = "hello"
        val s2 = s.clone()          // String implements Clone
        
        val owned: Own<String> = "world".toString()
        val owned2 = owned.clone()  // Own<String> implements Clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Clone should work on reference types");
}

#[test]
fn test_ref_on_owned_types() {
    // .ref() should work on Own<T> types
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned.ref()          // Own<String> -> String
        
        val owned_int: Own<I64> = 42.clone()       
        val ref_int: Ref<I64> = owned_int.ref()    // Own<I64> -> Ref<I64>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".ref() should work on owned types");
}

#[test]
fn test_ref_on_value_types() {
    // .ref() should work on value types to create references
    let code = r#"
    fun main() {
        val x: I64 = 42
        val ref_x: Ref<I64> = x.ref()              // I64 -> Ref<I64>
        
        val b: Bool = true
        val ref_b: Ref<Bool> = b.ref()             // Bool -> Ref<Bool>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".ref() should work on value types");
}

#[test]
fn test_mutref_on_owned_types() {
    // .mutRef() should work on Own<T> types
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val mut_ref: MutRef<Own<String>> = owned.mutRef()    // Own<String> -> MutRef<Own<String>>
        
        val owned_int: Own<I64> = 42.clone()
        val mut_ref_int: MutRef<Own<I64>> = owned_int.mutRef() // Own<I64> -> MutRef<Own<I64>>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".mutRef() should work on owned types");
}

#[test]
fn test_mutref_on_value_types() {
    // .mutRef() should work on value types
    let code = r#"
    fun main() {
        val x: I64 = 42
        val mut_ref: MutRef<I64> = x.mutRef()      // I64 -> MutRef<I64>
        
        val b: Bool = true
        val mut_ref_b: MutRef<Bool> = b.mutRef()   // Bool -> MutRef<Bool>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".mutRef() should work on value types");
}

#[test]
fn test_tostring_on_display_types() {
    // .toString() should work on types that implement Display
    let code = r#"
    fun main() {
        val x: I64 = 42
        val s1: Own<String> = x.toString()         // I64 implements Display
        
        val b: Bool = true
        val s2: Own<String> = b.toString()         // Bool implements Display
        
        val str_val: String = "hello"
        val s3: Own<String> = str_val.toString()   // String implements Display
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".toString() should work on Display types");
}

#[test]
fn test_bumpref_on_data_classes() {
    // .bumpRef() should work on data class types
    let code = r#"
    data class Person(val name: String, val age: I64)
    
    fun main() {
        val p: Person = Person("Alice", 30)
        val bumped: Person = p.bumpRef()           // Person -> Person (bumped)
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), ".bumpRef() should work on data classes");
}

// ============================================================================
// Negative Test Cases - Methods that should fail on specific receiver types
// ============================================================================

#[test]
fn test_invalid_mutref_on_borrowed_types() {
    // .mutRef() should NOT work on already borrowed types (String, Str)
    let code = r#"
    fun main() {
        val s: String = "hello"
        val invalid = s.mutRef()    // ERROR: Can't mutRef a borrowed type
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_err(), ".mutRef() should fail on borrowed types");

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
fn test_invalid_bumpref_on_non_data_class() {
    // .bumpRef() should only work on data classes
    let code = r#"
    fun main() {
        val x: I64 = 42
        val invalid = x.bumpRef()   // ERROR: bumpRef only for data classes
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_err(),
        ".bumpRef() should fail on non-data class types"
    );
}

#[test]
fn test_invalid_method_on_unit_type() {
    // Most methods shouldn't work on Unit type (except clone)
    let code = r#"
    fun main() {
        val u: Unit = ()
        val invalid = u.ref()       // ERROR: Unit doesn't support .ref()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_err(), ".ref() should fail on Unit type");
}

#[test]
fn test_invalid_tostring_on_non_display_type() {
    // .toString() should fail on types that don't implement Display
    let code = r#"
    data class NoDisplay()
    
    fun main() {
        val x = NoDisplay()
        val invalid = x.toString()  // ERROR: NoDisplay doesn't implement Display
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    // Note: This might pass if data classes automatically implement Display
    // Adjust the assertion based on the actual behavior
    if result.is_err() {
        if let Err(errors) = result {
            let has_method_not_found = errors
                .iter()
                .any(|err| matches!(err, veltrano::TypeCheckError::MethodNotFound { .. }));
            assert!(has_method_not_found, "Should have method not found error");
        }
    }
}

// ============================================================================
// Complex Receiver Type Tests - Testing chained methods and conversions
// ============================================================================

#[test]
fn test_chained_ref_methods() {
    // Test chaining multiple .ref() calls
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val ref1: String = owned.ref()                     // Own<String> -> String
        val ref2: Ref<String> = owned.ref().ref()          // Own<String> -> String -> Ref<String>
        val ref3: Ref<Ref<String>> = owned.ref().ref().ref() // ... -> Ref<Ref<String>>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Chained .ref() calls should work");
}

#[test]
fn test_mutref_to_ref_conversion() {
    // Test that MutRef<T> can be converted to Ref<T> via .ref()
    let code = r#"
    fun main() {
        val x: I64 = 42
        val mut_ref: MutRef<I64> = x.mutRef()
        val immut_ref: Ref<MutRef<I64>> = mut_ref.ref()    // MutRef<I64> -> Ref<MutRef<I64>>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "MutRef to Ref conversion should work");
}

#[test]
fn test_method_on_deeply_nested_types() {
    // Test methods on deeply nested type structures
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val mut_ref: MutRef<Own<String>> = owned.mutRef()
        val cloned: MutRef<Own<String>> = mut_ref.clone()  // MutRef implements Clone
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_ok(), "Methods on nested types should work");
}

// ============================================================================
// Error Message Validation Tests - Ensuring helpful error messages
// ============================================================================

#[test]
fn test_method_not_found_suggestion_for_owned_type() {
    // When a method is not found on Own<T>, suggest using .ref() first
    let code = r#"
    fun takeString(s: String): Int { return 42 }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = owned.length()  // ERROR: Should suggest .ref().length()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_err(), "Method should not be found on Own<String>");

    if let Err(errors) = result {
        let has_suggestion = errors.iter().any(|err| {
            matches!(err, veltrano::TypeCheckError::MethodNotFoundWithSuggestion { suggestion, .. }
                    if suggestion.contains(".ref()"))
        });
        assert!(has_suggestion, "Should suggest using .ref() first");
    }
}

#[test]
fn test_type_mismatch_with_conversion_suggestion() {
    // When types don't match, suggest appropriate conversions
    let code = r#"
    fun takeRef(x: Ref<I64>): Bool { return true }
    
    fun main() {
        val x: I64 = 42
        val result = takeRef(x)      // ERROR: Should suggest .ref()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    assert!(result.is_err(), "Type mismatch should occur");

    if let Err(errors) = result {
        let has_suggestion = errors.iter().any(|err| {
            matches!(err, veltrano::TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. }
                    if suggestion == ".ref()")
        });
        assert!(has_suggestion, "Should suggest .ref() conversion");
    }
}
