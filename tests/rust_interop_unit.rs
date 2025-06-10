use std::collections::HashMap;
use veltrano::rust_interop::*;
use veltrano::types::VeltranoType;

/// Unit tests for the Rust interop system
/// These tests use mocks and don't require external toolchain components

#[test]
fn test_error_handling_and_fallback() {
    let mut registry = DynamicRustRegistry::new();

    // Create queriers with different behaviors
    #[derive(Debug)]
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

    #[derive(Debug)]
    struct SuccessQuerier;
    impl RustQuerier for SuccessQuerier {
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, RustInteropError> {
            let mut types = HashMap::new();
            types.insert(
                "TestType".to_string(),
                TypeInfo {
                    name: "TestType".to_string(),
                    full_path: "test::TestType".to_string(),
                    kind: TypeKind::Struct,
                    generics: vec![],
                    methods: vec![MethodInfo {
                        name: "test_method".to_string(),
                        self_kind: SelfKind::Ref,
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
                    }],
                    fields: vec![],
                    variants: vec![],
                },
            );

            Ok(CrateInfo {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                functions: HashMap::new(),
                types,
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

    // Test that the fallback mechanism works - FailingQuerier fails first, then SuccessQuerier provides the type
    let result = registry.get_type("test::TestType");
    assert!(result.is_ok());
    let type_info = result.unwrap();
    assert!(type_info.is_some());
    let type_info = type_info.unwrap();
    assert_eq!(type_info.name, "TestType");
    assert_eq!(type_info.methods.len(), 1);
    assert_eq!(type_info.methods[0].name, "test_method");
}

#[test]
fn test_dynamic_registry_creation() {
    let registry = DynamicRustRegistry::new();
    assert!(registry.queriers.len() >= 2); // Should have at least StdLibQuerier and RustdocQuerier
    assert!(registry.queriers.len() <= 3); // May also have SynQuerier
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
    #[derive(Debug)]
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
fn test_registry_caching() {
    let mut registry = DynamicRustRegistry::new();

    // Create a custom querier that tracks how many times it's called
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug)]
    struct CountingQuerier {
        call_count: Rc<RefCell<usize>>,
    }

    impl RustQuerier for CountingQuerier {
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, RustInteropError> {
            *self.call_count.borrow_mut() += 1;

            let mut types = HashMap::new();
            types.insert(
                "CachedType".to_string(),
                TypeInfo {
                    name: "CachedType".to_string(),
                    full_path: "test::CachedType".to_string(),
                    kind: TypeKind::Struct,
                    generics: vec![],
                    methods: vec![],
                    fields: vec![],
                    variants: vec![],
                },
            );

            Ok(CrateInfo {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                functions: HashMap::new(),
                types,
                traits: HashMap::new(),
                trait_implementations: HashMap::new(),
            })
        }

        fn supports_crate(&self, crate_name: &str) -> bool {
            crate_name == "test"
        }

        fn priority(&self) -> u32 {
            250 // High priority to ensure it's used
        }
    }

    let call_count = Rc::new(RefCell::new(0));
    let querier = CountingQuerier {
        call_count: call_count.clone(),
    };

    registry.add_querier(Box::new(querier));

    // First call should query the crate
    let result1 = registry.get_type("test::CachedType").unwrap();
    assert!(result1.is_some());
    assert_eq!(*call_count.borrow(), 1);

    // Second call should use the cache
    let result2 = registry.get_type("test::CachedType").unwrap();
    assert!(result2.is_some());
    assert_eq!(*call_count.borrow(), 1); // Still 1, not 2!

    // Verify the results are the same
    assert_eq!(result1.unwrap().name, result2.unwrap().name);
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
        VeltranoType::i32()
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
        VeltranoType::own(VeltranoType::str())
    );

    assert_eq!(
        RustType::String.to_veltrano_type().unwrap(),
        VeltranoType::own(VeltranoType::string())
    );

    // Test references
    let rust_ref = RustType::Ref {
        lifetime: None,
        inner: Box::new(RustType::I32),
    };
    assert_eq!(
        rust_ref.to_veltrano_type().unwrap(),
        VeltranoType::ref_(VeltranoType::i32())
    );

    // Test mutable references
    let rust_mut_ref = RustType::MutRef {
        lifetime: None,
        inner: Box::new(RustType::String),
    };
    assert_eq!(
        rust_mut_ref.to_veltrano_type().unwrap(),
        VeltranoType::mut_ref(VeltranoType::own(VeltranoType::string()))
    );

    // Test Box
    let rust_box = RustType::Box(Box::new(RustType::I32));
    assert_eq!(
        rust_box.to_veltrano_type().unwrap(),
        VeltranoType::own(VeltranoType::boxed(VeltranoType::i32()))
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

    // Test that registries are created successfully
    assert!(dynamic_registry.queriers.len() >= 2); // Should have at least StdLibQuerier and RustdocQuerier
    assert!(dynamic_registry.queriers.len() <= 3); // May also have SynQuerier

    // Test that we can query traits from the standard library
    let clone_trait = dynamic_registry.get_trait("std::Clone");
    assert!(clone_trait.is_ok());
    let trait_info = clone_trait.unwrap();
    assert!(trait_info.is_some());
    let trait_info = trait_info.unwrap();
    assert_eq!(trait_info.name, "Clone");
    assert_eq!(trait_info.methods.len(), 1);
    assert_eq!(trait_info.methods[0].name, "clone");

    // The static registry should be initialized (though we can't test private fields directly)
    drop(static_registry); // Just to use it
}
