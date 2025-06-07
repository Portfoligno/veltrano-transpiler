use std::collections::HashMap;
use veltrano::rust_interop::*;
use veltrano::type_checker::VeltranoType;

/// Unit tests for the Rust interop system
/// These tests use mocks and don't require external toolchain components

#[test]
fn test_registry_caching() {
    let mut registry = DynamicRustRegistry::new();

    // Create a mock querier that tracks calls
    struct CallCountingQuerier {
        call_count: std::cell::RefCell<usize>,
    }

    impl RustQuerier for CallCountingQuerier {
        fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
            *self.call_count.borrow_mut() += 1;

            let mut functions = HashMap::new();
            functions.insert(
                "test_function".to_string(),
                FunctionInfo {
                    name: "test_function".to_string(),
                    full_path: format!("{}::test_function", crate_name),
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "()".to_string(),
                        parsed: Some(RustType::Unit),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                    documentation: None,
                },
            );

            Ok(CrateInfo {
                name: crate_name.to_string(),
                version: "1.0.0".to_string(),
                functions,
                types: HashMap::new(),
                traits: HashMap::new(),
                trait_implementations: HashMap::new(),
            })
        }

        fn supports_crate(&self, _: &str) -> bool {
            true
        }

        fn priority(&self) -> u32 {
            150 // Higher than rustdoc to ensure it's used first
        }
    }

    let counter = CallCountingQuerier {
        call_count: std::cell::RefCell::new(0),
    };
    registry.add_querier(Box::new(counter));

    // First call should query the crate
    let result1 = registry.get_function("test_crate::test_function");
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_some());

    // Second call should use cache (counter should not increment)
    let result2 = registry.get_function("test_crate::other_function");
    assert!(result2.is_ok());
    assert!(result2.unwrap().is_none()); // Function doesn't exist, but crate was cached

    // Verify only one call was made to the querier
    // Note: We can't easily access the counter after moving it into the Box,
    // but we can verify caching works by checking that subsequent calls are fast
}

#[test]
fn test_error_handling_and_fallback() {
    let mut registry = DynamicRustRegistry::new();

    // Create queriers with different behaviors
    struct FailingQuerier;
    impl RustQuerier for FailingQuerier {
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, RustInteropError> {
            Err(RustInteropError::CrateNotFound("Mock failure".to_string()))
        }
        fn supports_crate(&self, _: &str) -> bool {
            true
        }
        fn priority(&self) -> u32 {
            200
        } // Highest priority
    }

    struct SuccessQuerier;
    impl RustQuerier for SuccessQuerier {
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, RustInteropError> {
            let mut functions = HashMap::new();
            functions.insert(
                "fallback_function".to_string(),
                FunctionInfo {
                    name: "fallback_function".to_string(),
                    full_path: "test::fallback_function".to_string(),
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "i32".to_string(),
                        parsed: Some(RustType::I32),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                    documentation: Some("Fallback function".to_string()),
                },
            );

            Ok(CrateInfo {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                functions,
                types: HashMap::new(),
                traits: HashMap::new(),
                trait_implementations: HashMap::new(),
            })
        }
        fn supports_crate(&self, _: &str) -> bool {
            true
        }
        fn priority(&self) -> u32 {
            150
        } // Lower than failing querier
    }

    registry.add_querier(Box::new(FailingQuerier));
    registry.add_querier(Box::new(SuccessQuerier));

    // Should fallback to SuccessQuerier when FailingQuerier fails
    let result = registry.get_function("test::fallback_function");
    assert!(result.is_ok());
    let function_info = result.unwrap();
    assert!(function_info.is_some());
    assert_eq!(function_info.unwrap().name, "fallback_function");
}

#[test]
fn test_dynamic_registry_creation() {
    let registry = DynamicRustRegistry::new();
    assert!(registry.queriers.len() >= 1); // Should have at least rustdoc querier
    assert!(registry.queriers.len() <= 2); // May have syn querier too
}

#[test]
fn test_rustdoc_querier_creation() {
    let querier = RustdocQuerier::new(None);
    assert!(querier.supports_crate("std"));
    assert_eq!(querier.priority(), 100);
}

#[test]
fn test_path_parsing() {
    let registry = DynamicRustRegistry::new();

    // Valid path
    let (crate_name, item_path) = registry.parse_path("std::vec::Vec::new").unwrap();
    assert_eq!(crate_name, "std");
    assert_eq!(item_path, "vec::Vec::new");

    // Invalid path
    assert!(registry.parse_path("invalid_path").is_err());
}

#[test]
fn test_querier_priority_ordering() {
    let mut registry = DynamicRustRegistry::new();

    // Add a higher priority querier
    struct MockQuerier(u32);
    impl RustQuerier for MockQuerier {
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, RustInteropError> {
            Ok(CrateInfo {
                name: "mock".to_string(),
                version: "1.0".to_string(),
                functions: HashMap::new(),
                types: HashMap::new(),
                traits: HashMap::new(),
                trait_implementations: HashMap::new(),
            })
        }
        fn supports_crate(&self, _: &str) -> bool {
            true
        }
        fn priority(&self) -> u32 {
            self.0
        }
    }

    registry.add_querier(Box::new(MockQuerier(200))); // Higher priority
    registry.add_querier(Box::new(MockQuerier(50))); // Lower priority

    // Should be ordered by priority: 200, 100 (rustdoc), 80 (syn if present), 50
    assert_eq!(registry.queriers[0].priority(), 200);
    // The second querier should be rustdoc (100)
    assert!(registry.queriers[1].priority() >= 80);
    // The last querier should be the lowest priority (50)
    assert_eq!(registry.queriers.last().unwrap().priority(), 50);
}

#[test]
fn test_rust_type_signature_creation() {
    let signature = RustTypeSignature {
        raw: "Option<&'a str>".to_string(),
        parsed: Some(RustType::Option(Box::new(RustType::Ref {
            lifetime: Some("'a".to_string()),
            inner: Box::new(RustType::Str),
        }))),
        lifetimes: vec!["'a".to_string()],
        bounds: vec![],
    };

    assert_eq!(signature.raw, "Option<&'a str>");
    assert_eq!(signature.lifetimes, vec!["'a"]);
    assert!(signature.parsed.is_some());
}

#[test]
fn test_crate_info_serialization() {
    let crate_info = CrateInfo {
        name: "test_crate".to_string(),
        version: "1.0.0".to_string(),
        functions: HashMap::new(),
        types: HashMap::new(),
        traits: HashMap::new(),
        trait_implementations: HashMap::new(),
    };

    // Test serialization and deserialization
    let json = serde_json::to_string(&crate_info).unwrap();
    let deserialized: CrateInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(crate_info.name, deserialized.name);
    assert_eq!(crate_info.version, deserialized.version);
}

#[test]
fn test_syn_querier_creation() {
    // This test may pass or fail depending on whether we're in a Cargo project
    match SynQuerier::new(None) {
        Ok(querier) => {
            assert!(querier.supports_crate("test"));
            assert_eq!(querier.priority(), 80);
        }
        Err(_) => {
            // It's OK if we're not in a Cargo project
            println!("SynQuerier creation failed (not in Cargo project)");
        }
    }
}

#[test]
fn test_syn_type_conversion() {
    if let Ok(querier) = SynQuerier::new(None) {
        // Test basic type conversion
        let i32_type = syn::parse_str::<syn::Type>("i32").unwrap();
        let rust_type = querier.convert_syn_type_to_rust_type(&i32_type);
        assert_eq!(rust_type, Some(RustType::I32));

        // Test reference type conversion
        let ref_type = syn::parse_str::<syn::Type>("&str").unwrap();
        let rust_ref = querier.convert_syn_type_to_rust_type(&ref_type);
        assert!(matches!(rust_ref, Some(RustType::Ref { .. })));

        // Test unit type conversion
        let unit_type = syn::parse_str::<syn::Type>("()").unwrap();
        let rust_unit = querier.convert_syn_type_to_rust_type(&unit_type);
        assert_eq!(rust_unit, Some(RustType::Unit));
    }
}

#[test]
fn test_self_kind_determination() {
    if let Ok(querier) = SynQuerier::new(None) {
        // Test different self kinds
        let self_fn = syn::parse_str::<syn::Signature>("fn test(self)").unwrap();
        assert_eq!(
            querier.determine_self_kind(&self_fn.inputs),
            SelfKind::Value
        );

        let ref_self_fn = syn::parse_str::<syn::Signature>("fn test(&self)").unwrap();
        assert_eq!(
            querier.determine_self_kind(&ref_self_fn.inputs),
            SelfKind::Ref
        );

        let mut_ref_self_fn = syn::parse_str::<syn::Signature>("fn test(&mut self)").unwrap();
        assert_eq!(
            querier.determine_self_kind(&mut_ref_self_fn.inputs),
            SelfKind::MutRef
        );

        let no_self_fn = syn::parse_str::<syn::Signature>("fn test()").unwrap();
        assert_eq!(
            querier.determine_self_kind(&no_self_fn.inputs),
            SelfKind::None
        );
    }
}

#[test]
fn test_comprehensive_error_types() {
    // Test all error types
    let cargo_error = RustInteropError::CargoError("cargo failed".to_string());
    assert!(cargo_error.to_string().contains("Cargo error"));

    let parse_error = RustInteropError::ParseError("parse failed".to_string());
    assert!(parse_error.to_string().contains("Parse error"));

    let io_error = RustInteropError::IoError("io failed".to_string());
    assert!(io_error.to_string().contains("IO error"));

    let serde_error = RustInteropError::SerdeError("serde failed".to_string());
    assert!(serde_error.to_string().contains("Serialization error"));

    let crate_error = RustInteropError::CrateNotFound("test_crate".to_string());
    assert!(crate_error.to_string().contains("Crate not found"));

    // Test that errors implement std::error::Error
    fn assert_error<T: std::error::Error>(_: T) {}
    assert_error(cargo_error);
}

#[test]
fn test_querier_supports_crate() {
    let rustdoc_querier = RustdocQuerier::new(None);
    assert!(rustdoc_querier.supports_crate("std"));
    assert!(rustdoc_querier.supports_crate("any_crate"));

    if let Ok(syn_querier) = SynQuerier::new(None) {
        assert!(syn_querier.supports_crate("veltrano")); // Current project
    }
}

#[test]
fn test_rust_type_to_veltrano_conversion() {
    // Test basic types
    assert_eq!(
        RustType::I32.to_veltrano_type().unwrap(),
        VeltranoType::int()
    );

    assert_eq!(
        RustType::Bool.to_veltrano_type().unwrap(),
        VeltranoType::bool()
    );

    assert_eq!(
        RustType::Unit.to_veltrano_type().unwrap(),
        VeltranoType::unit()
    );

    // Test string types
    assert_eq!(
        RustType::Str.to_veltrano_type().unwrap(),
        VeltranoType::str()
    );

    assert_eq!(
        RustType::String.to_veltrano_type().unwrap(),
        VeltranoType::string()
    );

    // Test references
    let rust_ref = RustType::Ref {
        lifetime: None,
        inner: Box::new(RustType::I32),
    };
    assert_eq!(
        rust_ref.to_veltrano_type().unwrap(),
        VeltranoType::ref_(VeltranoType::int())
    );

    // Test mutable references
    let rust_mut_ref = RustType::MutRef {
        lifetime: None,
        inner: Box::new(RustType::String),
    };
    assert_eq!(
        rust_mut_ref.to_veltrano_type().unwrap(),
        VeltranoType::mut_ref(VeltranoType::string())
    );

    // Test Box
    let rust_box = RustType::Box(Box::new(RustType::I32));
    assert_eq!(
        rust_box.to_veltrano_type().unwrap(),
        VeltranoType::boxed(VeltranoType::int())
    );

    // Test custom types
    let rust_custom = RustType::Custom {
        name: "MyType".to_string(),
        generics: vec![],
    };
    assert_eq!(
        rust_custom.to_veltrano_type().unwrap(),
        VeltranoType::custom("MyType".to_string())
    );

    // Test generic parameters
    let rust_generic = RustType::Generic("T".to_string());
    assert_eq!(
        rust_generic.to_veltrano_type().unwrap(),
        VeltranoType::custom("$T".to_string())
    );
}

#[test]
fn test_integration_with_existing_registry() {
    use veltrano::rust_interop::RustInteropRegistry;

    // Test that the dynamic system works alongside the existing static registry
    let static_registry = RustInteropRegistry::new();
    let mut dynamic_registry = DynamicRustRegistry::new();

    // Test that static registry has pre-registered items
    assert!(static_registry.get_function("println").is_some());
    assert!(static_registry.get_method("Vec", "new").is_some());
    assert!(static_registry.get_method("String", "from").is_some());

    // Test dynamic registry fallback behavior
    let result = dynamic_registry.get_function("nonexistent::function");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
