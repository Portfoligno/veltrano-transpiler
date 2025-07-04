use veltrano::*;

mod common;

#[test]
fn test_basic_type_checking() {
    let code = r#"
    fun main() {
        val x: I64 = 42
        val y: Bool = true
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_ok(),
        "Type checking should succeed for basic program"
    );
}

#[test]
fn test_type_mismatch_detection() {
    let code = r#"
    fun processInt(x: I64): Bool {
        return true
    }
    
    fun main() {
        val result = processInt(true)  // Should cause type error
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_err(),
        "Type checking should fail for type mismatch"
    );

    if let Err(error) = result {
        // Check that we have a type mismatch error
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::TypeMismatch),
            "Should have a type mismatch error, got: {:?}",
            error
        );
    }
}

#[test]
fn test_variable_not_found() {
    let code = r#"
    fun main() {
        val x = undefinedVariable
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_err(),
        "Type checking should fail for undefined variable"
    );

    if let Err(error) = result {
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::UndefinedVariable),
            "Should have a variable not found error, got: {:?}",
            error
        );
    }
}

#[test]
fn test_ref_method_conversion() {
    let code = r#"
    fun takeString(s: String): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = takeString(owned.ref())  // Should work with explicit conversion
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    if let Err(error) = &result {
        eprintln!("Type check error: {:?}", error);
    }
    assert!(
        result.is_ok(),
        "Type checking should succeed with explicit .ref() conversion"
    );
}

#[test]
fn test_strict_type_checking_prevents_implicit_conversion() {
    let code = r#"
    fun takeString(s: String): Int {
        return 42
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val result = takeString(owned)  // Should fail without explicit conversion
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_err(),
        "Type checking should fail without explicit conversion"
    );
}

#[test]
fn test_error_analyzer_suggestions() {
    use veltrano::*;

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::string(), // String is naturally referenced in Veltrano
        actual: VeltranoType::own(VeltranoType::string()), // Own<String>
        location: error::SourceLocation::new(1, 1),
    };

    let enhanced = type_checker::error::ErrorAnalyzer::enhance_error(error);

    match enhanced {
        TypeCheckError::_TypeMismatchWithSuggestion { suggestion, .. } => {
            assert!(
                suggestion.contains(".ref()"),
                "Should suggest .ref() conversion"
            );
        }
        _ => panic!("Should have been enhanced with suggestion"),
    }
}

#[test]
fn test_shorthand_argument_type_checking() {
    let code = r#"
    data class Person(val name: String, val age: I64)
    
    fun main() {
        val name: String = "Alice".toString().ref()
        val age: String = "not a number".toString().ref()  // Wrong type for age field
        val person = Person(.name, .age)  // Should fail type checking
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_err(),
        "Type checking should fail for shorthand argument with wrong type"
    );

    if let Err(error) = result {
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::TypeMismatch),
            "Should have a type mismatch error for shorthand argument, got: {:?}",
            error
        );
    }
}

#[test]
fn test_shorthand_argument_field_not_found() {
    let code = r#"
    data class Person(val name: String, val age: I64)
    
    fun main() {
        val name: String = "Alice".toString().ref()
        val person = Person(.name, .undefinedField)  // Should fail - field not found
    }
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let result = common::parse_and_type_check(code, config).map(|_| ());

    assert!(
        result.is_err(),
        "Type checking should fail for shorthand argument with undefined field"
    );

    if let Err(error) = result {
        assert!(
            matches!(error.kind, veltrano::error::ErrorKind::TypeError),
            "Should have a field not found error for shorthand argument, got: {:?}",
            error
        );
    }
}
