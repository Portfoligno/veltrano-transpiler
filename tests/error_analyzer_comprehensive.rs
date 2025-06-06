// Error analyzer comprehensive tests updated for new VeltranoType structure

#![cfg(test)]

use veltrano::*;

#[test]
fn test_owned_to_borrowed_suggestion() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::string(), // String (naturally referenced)
        actual: VeltranoType::own(VeltranoType::string()), // Own<String>
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref()");
        }
        _ => panic!("Should have been enhanced with .ref() suggestion"),
    }
}

#[test]
fn test_owned_string_to_str_suggestion() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::str(),                     // Str
        actual: VeltranoType::own(VeltranoType::string()), // Own<String>
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref().ref()");
        }
        _ => panic!("Should have been enhanced with .ref().ref() suggestion"),
    }
}

#[test]
fn test_borrowed_string_to_str_suggestion() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::str(),  // Str
        actual: VeltranoType::string(), // String (naturally referenced)
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref()");
        }
        _ => panic!("Should have been enhanced with .ref() suggestion"),
    }
}

#[test]
fn test_mutref_to_borrowed_suggestion() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::ref_type(VeltranoType::string()), // Ref<String>
        actual: VeltranoType::mut_ref(VeltranoType::string()),    // MutRef<String>
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref()");
        }
        _ => panic!("Should have been enhanced with .ref() suggestion"),
    }
}

#[test]
fn test_vec_to_slice_suggestion() {
    let inner_type = VeltranoType::int();

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::ref_type(inner_type.clone()), // Ref<Int> (slice-like)
        actual: VeltranoType::vec(inner_type),                // Vec<Int>
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".toSlice()");
        }
        _ => panic!("Should have been enhanced with .toSlice() suggestion"),
    }
}

#[test]
fn test_array_to_slice_suggestion() {
    let inner_type = VeltranoType::int();

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::ref_type(inner_type.clone()), // Ref<Int> (slice-like)
        actual: VeltranoType::array(inner_type, 3),           // Array<Int, 3>
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".toSlice()");
        }
        _ => panic!("Should have been enhanced with .toSlice() suggestion"),
    }
}

#[test]
fn test_owned_array_to_slice_suggestion() {
    let inner_type = VeltranoType::int();

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::ref_type(inner_type.clone()), // Ref<Int> (slice-like)
        actual: VeltranoType::own(VeltranoType::array(inner_type, 3)), // Own<Array<Int, 3>>
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatchWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref().toSlice()");
        }
        _ => panic!("Should have been enhanced with .ref().toSlice() suggestion"),
    }
}

#[test]
fn test_method_not_found_suggestion() {
    let receiver_type = VeltranoType::own(VeltranoType::string()); // Own<String>

    let error = TypeCheckError::MethodNotFound {
        receiver_type: receiver_type.clone(),
        method: "length".to_string(),
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::MethodNotFoundWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref().length()");
        }
        _ => panic!("Should have been enhanced with method suggestion"),
    }
}

#[test]
fn test_field_not_found_suggestion() {
    let object_type = VeltranoType::own(VeltranoType::custom("Person".to_string())); // Own<Person>

    let error = TypeCheckError::FieldNotFound {
        object_type: object_type.clone(),
        field: "name".to_string(),
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::FieldNotFoundWithSuggestion { suggestion, .. } => {
            assert_eq!(suggestion, ".ref().name");
        }
        _ => panic!("Should have been enhanced with field suggestion"),
    }
}

#[test]
fn test_no_suggestion_for_unrelated_types() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType::int(), // Int
        actual: VeltranoType::bool(),  // Bool
        location: SourceLocation {
            file: "test.vl".to_string(),
            line: 1,
            column: 1,
            source_line: "test".to_string(),
        },
    };

    let analyzer = ErrorAnalyzer;
    let enhanced = analyzer.enhance_error(error);

    match enhanced {
        TypeCheckError::TypeMismatch { .. } => {
            // Should remain as TypeMismatch without suggestion
        }
        _ => panic!("Should remain as TypeMismatch without suggestion for unrelated types"),
    }
}
