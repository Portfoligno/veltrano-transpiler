use veltrano::config::Config;

mod common;
use common::parse_and_type_check;

#[test]
fn test_chained_ref_conversions() {
    let code = r#"
    fun takeString(s: String): Int {
        return 42
    }
    
    fun takeRefString(s: Ref<String>): Int {
        return 43
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result1 = takeString(owned.ref())        // Own<String> → String
        val result2 = takeRefString(owned.ref().ref()) // Own<String> → String → Ref<String>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Type check error: {:?}", error);
        }
    }

    assert!(result.is_ok(), "Chained .ref() conversions should work");
}

#[test]
fn test_mutref_conversion() {
    let code = r#"
    fun takeMutRefOwned(s: MutRef<Own<String>>): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = takeMutRefOwned(owned.mutRef())  // Own<String> → MutRef<Own<String>>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Type check error: {:?}", error);
        }
    }

    assert!(result.is_ok(), "MutRef conversion should work");
}

#[test]
fn test_mutref_to_immutable_conversion() {
    let code = r#"
    fun takeRefMutRef(s: Ref<MutRef<Own<String>>>): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val mutRef: MutRef<Own<String>> = owned.mutRef()
        val result = takeRefMutRef(mutRef.ref())  // MutRef<Own<String>> → Ref<MutRef<Own<String>>>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Type check error: {:?}", error);
        }
    }

    assert!(result.is_ok(), "MutRef to immutable conversion should work");
}

#[test]
fn test_clone_preserves_type() {
    // With explicit conversion enforcement, Own<String> cannot directly call clone()
    // Must use .ref().clone() for explicit conversion
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val ref_string: String = owned.ref()
        val cloned: String = ref_string.clone()  // String.clone() -> String
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(result.is_ok(), "Clone should preserve the exact type with explicit conversion");
}

#[test]
fn test_double_ref_on_int_is_allowed() {
    let code = r#"
    fun takeRefRefInt(x: Ref<Ref<I64>>): I64 {
        return 42
    }
    
    fun main() {
        val x: I64 = 42
        val borrowed = x.ref()        // I64 → Ref<I64>
        val doubleBorrowed = borrowed.ref()  // Ref<I64> → Ref<Ref<I64>>
        val result = takeRefRefInt(doubleBorrowed)
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(
        result.is_ok(),
        "Double ref on I64 should be allowed: I64 → Ref<I64> → Ref<Ref<I64>>"
    );
}

#[test]
fn test_invalid_mutref_on_borrowed_type() {
    let code = r#"
    fun main() {
        val s: String = "hello"
        val invalid = s.mutRef()  // Should fail - can't .mutRef() a borrowed type
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(
        result.is_err(),
        "Should fail when trying to .mutRef() a borrowed type"
    );
}

#[test]
fn test_comprehensive_type_combinations() {
    let code = r#"
    fun processString(s: String): Int { return 2 }
    fun processRefString(s: Ref<String>): Int { return 1 }
    fun processMutRefOwned(s: MutRef<Own<String>>): Int { return 3 }
    fun processRefMutRefOwned(s: Ref<MutRef<Own<String>>>): Int { return 4 }
    fun processRefRefMutRefOwned(s: Ref<Ref<MutRef<Own<String>>>>): Int { return 5 }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        
        // All these should work with proper conversions
        val result1 = processRefString(owned.ref().ref())        // Own<String> → String → Ref<String>
        val result2 = processString(owned.ref())                 // Own<String> → String  
        val result3 = processMutRefOwned(owned.mutRef())         // Own<String> → MutRef<Own<String>>
        
        // Test mutRef conversion to immutable  
        val mutRef: MutRef<Own<String>> = owned.mutRef()
        val result4 = processRefMutRefOwned(mutRef.ref())        // MutRef<Own<String>> → Ref<MutRef<Own<String>>>
        val result5 = processRefRefMutRefOwned(mutRef.ref().ref()) // MutRef<Own<String>> → Ref<MutRef<Own<String>>> → Ref<Ref<MutRef<Own<String>>>>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());

    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Type check error: {:?}", error);
        }
    }

    assert!(
        result.is_ok(),
        "All explicit conversions should work correctly"
    );
}
