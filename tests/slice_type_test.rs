//! Tests for the Slice type constructor

use veltrano::types::{TypeConstructor, VeltranoType};

#[test]
fn test_slice_type_construction() {
    // Test creating a Slice<I64> type
    let slice_i64 = VeltranoType::slice(VeltranoType::i64());
    assert!(matches!(slice_i64.constructor, TypeConstructor::Slice));
    assert_eq!(slice_i64.args.len(), 1);
    assert!(matches!(
        slice_i64.args[0].constructor,
        TypeConstructor::I64
    ));
}

#[test]
fn test_slice_of_string() {
    // Test creating a Slice<String> type
    let slice_string = VeltranoType::slice(VeltranoType::string());
    assert!(matches!(slice_string.constructor, TypeConstructor::Slice));
    assert_eq!(slice_string.args.len(), 1);
    assert!(matches!(
        slice_string.args[0].constructor,
        TypeConstructor::String
    ));
}

#[test]
fn test_slice_of_vec() {
    // Test creating a Slice<Vec<I32>> type
    let slice_vec = VeltranoType::slice(VeltranoType::vec(VeltranoType::i32()));
    assert!(matches!(slice_vec.constructor, TypeConstructor::Slice));
    assert_eq!(slice_vec.args.len(), 1);
    if let Some(inner) = slice_vec.inner() {
        assert!(matches!(inner.constructor, TypeConstructor::Vec));
    } else {
        panic!("Expected inner type");
    }
}

#[test]
fn test_nested_slice() {
    // Test creating a Slice<Slice<I64>> type (slice of slices)
    let nested_slice = VeltranoType::slice(VeltranoType::slice(VeltranoType::i64()));
    assert!(matches!(nested_slice.constructor, TypeConstructor::Slice));
    assert_eq!(nested_slice.args.len(), 1);
    if let Some(inner) = nested_slice.inner() {
        assert!(matches!(inner.constructor, TypeConstructor::Slice));
    } else {
        panic!("Expected inner type");
    }
}
