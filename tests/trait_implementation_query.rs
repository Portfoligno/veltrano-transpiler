use veltrano::rust_interop::DynamicRustRegistry;

#[test]
fn test_built_in_type_trait_implementations() {
    let mut registry = DynamicRustRegistry::new();

    // Test primitive types implement Clone
    assert!(registry._type_implements_trait("i32", "Clone").unwrap());
    assert!(registry._type_implements_trait("bool", "Clone").unwrap());
    assert!(registry._type_implements_trait("()", "Clone").unwrap());

    // Test String implements Clone and ToString
    assert!(registry._type_implements_trait("String", "Clone").unwrap());
    assert!(registry
        ._type_implements_trait("String", "ToString")
        .unwrap());
    assert!(registry
        ._type_implements_trait("String", "Display")
        .unwrap());

    // Test &str doesn't implement Clone but does implement Display
    assert!(!registry._type_implements_trait("&str", "Clone").unwrap());
    assert!(registry._type_implements_trait("&str", "Display").unwrap());
    assert!(registry._type_implements_trait("&str", "ToString").unwrap());
}

#[test]
fn test_get_implemented_traits() {
    let mut registry = DynamicRustRegistry::new();

    // Test primitive type traits
    let i32_traits = registry.get_implemented_traits("i32").unwrap();
    assert!(i32_traits.contains(&"Clone".to_string()));
    assert!(i32_traits.contains(&"Copy".to_string()));
    assert!(i32_traits.contains(&"Debug".to_string()));

    // Test String traits
    let string_traits = registry.get_implemented_traits("String").unwrap();
    assert!(string_traits.contains(&"Clone".to_string()));
    assert!(string_traits.contains(&"Display".to_string()));
    assert!(string_traits.contains(&"ToString".to_string()));
    assert!(!string_traits.contains(&"Copy".to_string())); // String is not Copy
}

// TODO: Add tests for custom types once we have test data with trait implementations
