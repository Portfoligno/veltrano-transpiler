use veltrano::*;

#[test]
fn test_owned_to_borrowed_suggestion() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType {
            base: VeltranoBaseType::String,
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::String,
            ownership: Ownership::Owned,
            mutability: Mutability::Immutable,
        },
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
        expected: VeltranoType {
            base: VeltranoBaseType::Str,
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::String,
            ownership: Ownership::Owned,
            mutability: Mutability::Immutable,
        },
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
        expected: VeltranoType {
            base: VeltranoBaseType::Str,
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::String,
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
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
        expected: VeltranoType {
            base: VeltranoBaseType::String,
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::String,
            ownership: Ownership::MutBorrowed,
            mutability: Mutability::Mutable,
        },
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
    let inner_type = VeltranoType {
        base: VeltranoBaseType::Int,
        ownership: Ownership::Owned,
        mutability: Mutability::Immutable,
    };

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType {
            base: VeltranoBaseType::Slice(Box::new(inner_type.clone())),
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::Vec(Box::new(inner_type)),
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
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
    let inner_type = VeltranoType {
        base: VeltranoBaseType::Int,
        ownership: Ownership::Owned,
        mutability: Mutability::Immutable,
    };

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType {
            base: VeltranoBaseType::Slice(Box::new(inner_type.clone())),
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::Array(Box::new(inner_type), 3),
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
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
    let inner_type = VeltranoType {
        base: VeltranoBaseType::Int,
        ownership: Ownership::Owned,
        mutability: Mutability::Immutable,
    };

    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType {
            base: VeltranoBaseType::Slice(Box::new(inner_type.clone())),
            ownership: Ownership::Borrowed,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::Array(Box::new(inner_type), 3),
            ownership: Ownership::Owned,
            mutability: Mutability::Immutable,
        },
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
    let receiver_type = VeltranoType {
        base: VeltranoBaseType::String,
        ownership: Ownership::Owned,
        mutability: Mutability::Immutable,
    };

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
            assert!(suggestion.contains(".ref().length()"));
        }
        _ => panic!("Should have been enhanced with method suggestion"),
    }
}

#[test]
fn test_field_not_found_suggestion() {
    let object_type = VeltranoType {
        base: VeltranoBaseType::Custom("Person".to_string()),
        ownership: Ownership::Owned,
        mutability: Mutability::Immutable,
    };

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
            assert!(suggestion.contains(".ref().name"));
        }
        _ => panic!("Should have been enhanced with field suggestion"),
    }
}

#[test]
fn test_no_suggestion_for_unrelated_types() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType {
            base: VeltranoBaseType::Int,
            ownership: Ownership::Owned,
            mutability: Mutability::Immutable,
        },
        actual: VeltranoType {
            base: VeltranoBaseType::Bool,
            ownership: Ownership::Owned,
            mutability: Mutability::Immutable,
        },
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
