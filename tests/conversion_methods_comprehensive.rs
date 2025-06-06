use veltrano::config::Config;

mod common;
use common::parse_and_type_check;

#[test]
fn test_chained_ref_conversions() {
    let code = r#"
    fun takeStr(s: Str): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = takeStr(owned.ref().ref())  // Own<String> → String → Str
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(result.is_ok(), "Chained .ref() conversions should work");
}

#[test]
fn test_mutref_conversion() {
    let code = r#"
    fun takeMutRef(s: MutRef<String>): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = takeMutRef(owned.mutRef())  // Own<String> → MutRef<String>
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(result.is_ok(), "MutRef conversion should work");
}

#[test]
fn test_mutref_to_immutable_conversion() {
    let code = r#"
    fun takeString(s: String): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val mutRef: MutRef<String> = owned.mutRef()
        val result = takeString(mutRef.ref())  // MutRef<String> → String
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(result.is_ok(), "MutRef to immutable conversion should work");
}

#[test]
fn test_clone_preserves_type() {
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val cloned: Own<String> = owned.clone()
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(result.is_ok(), "Clone should preserve the exact type");
}

#[test]
fn test_invalid_double_ref_on_borrowed_int() {
    let code = r#"
    fun main() {
        val x: Int = 42
        val borrowed = x.ref()  // This should work: Int → Int (borrowed)
        val invalid = borrowed.ref()  // This should fail - Int doesn't support String→Str conversion
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = parse_and_type_check(code, config).map(|_| ());
    assert!(
        result.is_err(),
        "Should fail when trying to .ref() a borrowed Int (not String→Str conversion)"
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
    fun processStr(s: Str): Int { return 1 }
    fun processString(s: String): Int { return 2 }
    fun processMutRef(s: MutRef<String>): Int { return 3 }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        
        // All these should work with proper conversions
        val result1 = processStr(owned.ref().ref())        // Own<String> → Str
        val result2 = processString(owned.ref())           // Own<String> → String  
        val result3 = processMutRef(owned.mutRef())        // Own<String> → MutRef<String>
        
        // Test mutRef conversion to immutable
        val mutRef: MutRef<String> = owned.mutRef()
        val result4 = processString(mutRef.ref())          // MutRef<String> → String
        val result5 = processStr(mutRef.ref().ref())       // MutRef<String> → Str
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
