use std::collections::HashMap;
use veltrano::error::VeltranoError;
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
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, VeltranoError> {
            Err(VeltranoError::from(RustInteropError::CrateNotFound(
                "Mock failure".to_string(),
            )))
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
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, VeltranoError> {
            let mut types = HashMap::new();
            types.insert(
                "TestType".to_string(),
                TypeInfo {
                    name: "TestType".to_string(),
                    path: RustPath::Type(RustTypePath(
                        RustModulePath("test".into(), vec![]),
                        vec!["TestType".to_string()],
                    )),
                    kind: TypeKind::Struct,
                    generics: vec![],
                    methods: vec![MethodInfo {
                        name: "test_method".to_string(),
                        self_kind: SelfKind::Ref(None),
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
    assert_eq!(querier.priority(), 90);
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
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, VeltranoError> {
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
        fn query_crate(&mut self, _: &str) -> Result<CrateInfo, VeltranoError> {
            *self.call_count.borrow_mut() += 1;

            let mut types = HashMap::new();
            types.insert(
                "CachedType".to_string(),
                TypeInfo {
                    name: "CachedType".to_string(),
                    path: RustPath::Type(RustTypePath(
                        RustModulePath("test".into(), vec![]),
                        vec!["CachedType".to_string()],
                    )),
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
            SelfKind::Ref(None)
        );

        let mut_ref_self_fn = syn::parse_str::<syn::Signature>("fn test(&mut self)").unwrap();
        assert_eq!(
            querier.determine_self_kind(&mut_ref_self_fn.inputs),
            SelfKind::MutRef(None)
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
    assert!(rustdoc_querier.supports_crate("core"));

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

#[test]
fn test_associated_type_extraction() {
    // Create a test trait with associated types
    let trait_code = r#"
        pub trait MyIterator {
            type Item;
            type Error;
            
            fn next(&mut self) -> Option<Self::Item>;
            fn error(&self) -> Option<Self::Error>;
        }
    "#;

    // Parse the trait
    let file = syn::parse_file(trait_code).unwrap();
    if let syn::Item::Trait(trait_item) = &file.items[0] {
        let syn_querier = SynQuerier::new(None).unwrap();
        let trait_info = syn_querier.extract_trait(trait_item, "test").unwrap();

        // Check that associated types were extracted
        assert_eq!(trait_info.associated_types.len(), 2);
        assert!(trait_info.associated_types.contains(&"Item".to_string()));
        assert!(trait_info.associated_types.contains(&"Error".to_string()));

        // Check that methods were also extracted
        assert_eq!(trait_info.methods.len(), 2);
        assert_eq!(trait_info.methods[0].name, "next");
        assert_eq!(trait_info.methods[1].name, "error");
    } else {
        panic!("Expected trait item");
    }
}

#[test]
fn test_generic_parameter_defaults() {
    // Create test structures with generic defaults
    let code = r#"
        pub struct Container<T = String> {
            value: T,
        }
        
        pub struct MultiParam<T = i32, U = Vec<String>, V: Clone = HashMap<String, i32>> {
            first: T,
            second: U,
            third: V,
        }
        
        pub fn with_default<T = bool>() -> T {
            unimplemented!()
        }
    "#;

    let file = syn::parse_file(code).unwrap();
    let syn_querier = SynQuerier::new(None).unwrap();

    // Test struct with single default
    if let syn::Item::Struct(struct_item) = &file.items[0] {
        let type_info = syn_querier.extract_struct(struct_item, "test").unwrap();
        assert_eq!(type_info.generics.len(), 1);
        assert_eq!(type_info.generics[0].name, "T");
        assert_eq!(type_info.generics[0].default, Some("String".to_string()));
    } else {
        panic!("Expected struct item");
    }

    // Test struct with multiple defaults and bounds
    if let syn::Item::Struct(struct_item) = &file.items[1] {
        let type_info = syn_querier.extract_struct(struct_item, "test").unwrap();
        assert_eq!(type_info.generics.len(), 3);

        assert_eq!(type_info.generics[0].name, "T");
        assert_eq!(type_info.generics[0].default, Some("i32".to_string()));
        assert_eq!(type_info.generics[0].bounds.len(), 0);

        assert_eq!(type_info.generics[1].name, "U");
        assert_eq!(
            type_info.generics[1].default,
            Some("Vec < String >".to_string())
        );
        assert_eq!(type_info.generics[1].bounds.len(), 0);

        assert_eq!(type_info.generics[2].name, "V");
        assert_eq!(
            type_info.generics[2].default,
            Some("HashMap < String , i32 >".to_string())
        );
        assert_eq!(type_info.generics[2].bounds.len(), 1);
        assert_eq!(type_info.generics[2].bounds[0], "Clone");
    } else {
        panic!("Expected struct item");
    }

    // Test function with default
    if let syn::Item::Fn(fn_item) = &file.items[2] {
        let fn_info = syn_querier.extract_function(fn_item, "test").unwrap();
        assert_eq!(fn_info.generics.len(), 1);
        assert_eq!(fn_info.generics[0].name, "T");
        assert_eq!(fn_info.generics[0].default, Some("bool".to_string()));
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_self_parameter_lifetime_extraction() {
    // Create test code with various self parameter lifetimes
    let code = r#"
        impl MyStruct {
            pub fn borrow(&self) -> &str {
                &self.data
            }
            
            pub fn borrow_with_lifetime<'a>(&'a self) -> &'a str {
                &self.data
            }
            
            pub fn borrow_mut(&mut self) -> &mut str {
                &mut self.data
            }
            
            pub fn borrow_mut_with_lifetime<'a>(&'a mut self) -> &'a mut str {
                &mut self.data
            }
            
            pub fn consume(self) -> String {
                self.data
            }
            
            pub fn static_method() -> String {
                String::new()
            }
        }
    "#;

    let file = syn::parse_file(code).unwrap();
    let syn_querier = SynQuerier::new(None).unwrap();

    if let syn::Item::Impl(impl_item) = &file.items[0] {
        // Extract methods
        let methods: Vec<_> = impl_item
            .items
            .iter()
            .filter_map(|item| {
                if let syn::ImplItem::Fn(method) = item {
                    Some((
                        method.sig.ident.to_string(),
                        syn_querier.extract_self_kind(&method.sig),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Check each method's self kind
        assert_eq!(methods.len(), 6);

        // &self without explicit lifetime
        assert_eq!(methods[0].0, "borrow");
        assert!(matches!(methods[0].1, SelfKind::Ref(None)));

        // &'a self with explicit lifetime
        assert_eq!(methods[1].0, "borrow_with_lifetime");
        assert!(matches!(methods[1].1, SelfKind::Ref(Some(ref lt)) if lt == "a"));

        // &mut self without explicit lifetime
        assert_eq!(methods[2].0, "borrow_mut");
        assert!(matches!(methods[2].1, SelfKind::MutRef(None)));

        // &'a mut self with explicit lifetime
        assert_eq!(methods[3].0, "borrow_mut_with_lifetime");
        assert!(matches!(methods[3].1, SelfKind::MutRef(Some(ref lt)) if lt == "a"));

        // self (by value)
        assert_eq!(methods[4].0, "consume");
        assert!(matches!(methods[4].1, SelfKind::Value));

        // no self (static method)
        assert_eq!(methods[5].0, "static_method");
        assert!(matches!(methods[5].1, SelfKind::None));
    } else {
        panic!("Expected impl item");
    }
}

#[test]
fn test_where_clause_extraction() {
    // Test various where clause patterns
    let trait_code = r#"
        pub trait MyTrait<T, U> 
        where 
            T: Clone + Send,
            U: Into<String> + Default,
            Self: Sized,
        {
            fn process(&self, value: T) -> U;
        }
        
        pub trait ComplexTrait<'a, T: 'a> 
        where 
            T: Iterator<Item = &'a str>,
            T::Item: AsRef<str>,
            for<'b> &'b T: IntoIterator,
        {
            fn iterate(&self, iter: T);
        }
        
        pub trait SimpleTraitNoWhere {
            fn simple(&self);
        }
    "#;

    let file = syn::parse_file(trait_code).unwrap();
    let syn_querier = SynQuerier::new(None).unwrap();

    // Test MyTrait with simple where clauses
    if let syn::Item::Trait(trait_item) = &file.items[0] {
        let trait_info = syn_querier.extract_trait(trait_item, "test").unwrap();

        assert_eq!(trait_info.name, "MyTrait");
        assert!(trait_info.where_clause.is_some());

        let where_predicates = trait_info.where_clause.unwrap();
        assert_eq!(where_predicates.len(), 3);

        // Check that predicates are captured (exact format may vary)
        assert!(where_predicates[0].contains("T") && where_predicates[0].contains("Clone"));
        assert!(where_predicates[1].contains("U") && where_predicates[1].contains("Into"));
        assert!(where_predicates[2].contains("Self") && where_predicates[2].contains("Sized"));
    }

    // Test ComplexTrait with lifetime bounds and HRTB
    if let syn::Item::Trait(trait_item) = &file.items[1] {
        let trait_info = syn_querier.extract_trait(trait_item, "test").unwrap();

        assert_eq!(trait_info.name, "ComplexTrait");
        assert!(trait_info.where_clause.is_some());

        let where_predicates = trait_info.where_clause.unwrap();
        assert_eq!(where_predicates.len(), 3);

        // Verify complex bounds are captured
        assert!(where_predicates[0].contains("Iterator"));
        assert!(where_predicates[1].contains("Item"));
        assert!(where_predicates[2].contains("for"));
    }

    // Test SimpleTraitNoWhere without where clause
    if let syn::Item::Trait(trait_item) = &file.items[2] {
        let trait_info = syn_querier.extract_trait(trait_item, "test").unwrap();

        assert_eq!(trait_info.name, "SimpleTraitNoWhere");
        assert!(trait_info.where_clause.is_none());
    }
}

#[test]
fn test_rustdoc_function_conversion() {
    use serde_json::json;
    use veltrano::rust_interop::RustdocQuerier;

    // Create a comprehensive mock rustdoc JSON structure
    let rustdoc_json = json!({
        "crate_name": "test_crate",
        "crate_version": "0.1.0",
        "paths": {
            "0:0": {
                "crate_id": 0,
                "path": ["test_crate", "utils", "math", "test_function"],
                "kind": "function"
            },
            "0:1": {
                "crate_id": 0,
                "path": ["test_crate", "MY_CONSTANT"],
                "kind": "constant"
            },
            "0:2": {
                "crate_id": 1,
                "path": ["std", "vec", "Vec", "push"],
                "kind": "function"
            }
        },
        "external_crates": {
            "1": {
                "name": "std",
                "html_root_url": "https://doc.rust-lang.org/stable/"
            }
        },
        "index": {
            "0:0": {
                "id": "0:0",
                "crate_id": 0,
                "name": "test_function",
                "kind": "function",
                "inner": {
                    "sig": {
                        "inputs": [
                            ["x", "i32"],
                            ["y", "&str"],
                            ["z", "Vec<T>"]
                        ],
                        "output": "Result<String, Box<dyn Error>>"
                    },
                    "generics": {
                        "params": [
                            {
                                "name": "T",
                                "bounds": ["Clone", "Debug"],
                                "default": null
                            }
                        ]
                    },
                    "header": {
                        "is_const": false,
                        "is_unsafe": true,
                        "is_async": false
                    }
                }
            },
            "0:1": {
                "id": "0:1",
                "crate_id": 0,
                "name": "MY_CONSTANT",
                "kind": "constant",
                "inner": {
                    "sig": {
                        "inputs": [],
                        "output": "i32"
                    },
                    "generics": {
                        "params": []
                    },
                    "header": {
                        "is_const": true,
                        "is_unsafe": false,
                        "is_async": false
                    }
                }
            },
            "0:2": {
                "id": "0:2",
                "crate_id": 1,
                "name": "push",
                "kind": "function",
                "inner": {
                    "sig": {
                        "inputs": [
                            ["self", "&mut Vec<T>"],
                            ["value", "T"]
                        ],
                        "output": null
                    },
                    "generics": {
                        "params": [
                            {
                                "name": "T",
                                "bounds": [],
                                "default": null
                            }
                        ]
                    },
                    "header": {
                        "is_const": false,
                        "is_unsafe": false,
                        "is_async": false
                    }
                }
            }
        }
    });

    // Test that we can parse and convert the rustdoc JSON
    let querier = RustdocQuerier::new(None);
    let crate_info = querier
        .convert_rustdoc_to_crate_info(serde_json::from_value(rustdoc_json).unwrap())
        .unwrap();

    // Verify crate name
    assert_eq!(crate_info.name, "test_crate");
    assert_eq!(crate_info.version, "0.1.0");

    // Test function conversion with module path
    let test_func = crate_info.functions.get("test_function").unwrap();
    assert_eq!(test_func.name, "test_function");
    assert!(test_func.is_unsafe);
    assert!(!test_func.is_const);

    // Verify module path extraction
    match &test_func.path {
        RustPath::ModuleItem(module_path, name, ItemKind::Function) => {
            assert_eq!(name, "test_function");
            assert_eq!((module_path.0).0, "test_crate");
            assert_eq!(module_path.1, vec!["utils", "math"]);
        }
        _ => panic!("Expected ModuleItem with Function kind"),
    }

    // Verify parameters with parsed types
    assert_eq!(test_func.parameters.len(), 3);
    assert_eq!(test_func.parameters[0].name, "x");
    assert_eq!(test_func.parameters[0].param_type.raw, "i32");
    assert!(matches!(
        test_func.parameters[0].param_type.parsed,
        Some(RustType::I32)
    ));

    assert_eq!(test_func.parameters[1].name, "y");
    assert_eq!(test_func.parameters[1].param_type.raw, "&str");
    assert!(matches!(
        test_func.parameters[1].param_type.parsed,
        Some(RustType::Ref {
            lifetime: None,
            inner: _
        })
    ));

    assert_eq!(test_func.parameters[2].name, "z");
    assert_eq!(test_func.parameters[2].param_type.raw, "Vec<T>");
    assert!(matches!(
        test_func.parameters[2].param_type.parsed,
        Some(RustType::Vec(_))
    ));

    // Verify return type parsing
    assert_eq!(test_func.return_type.raw, "Result<String, Box<dyn Error>>");
    assert!(matches!(
        test_func.return_type.parsed,
        Some(RustType::Result { .. })
    ));

    // Verify generics
    assert_eq!(test_func.generics.len(), 1);
    assert_eq!(test_func.generics[0].name, "T");
    assert_eq!(test_func.generics[0].bounds, vec!["Clone", "Debug"]);

    // Test constant conversion
    let constant = crate_info.functions.get("MY_CONSTANT").unwrap();
    assert_eq!(constant.name, "MY_CONSTANT");
    match &constant.path {
        RustPath::ModuleItem(module_path, name, ItemKind::Constant) => {
            assert_eq!(name, "MY_CONSTANT");
            assert_eq!((module_path.0).0, "test_crate");
            assert!(module_path.1.is_empty()); // Root level constant
        }
        _ => panic!("Expected ModuleItem with Constant kind"),
    }

    // Test external crate function
    let external_func = crate_info.functions.get("push").unwrap();
    match &external_func.path {
        RustPath::ModuleItem(module_path, name, ItemKind::Function) => {
            assert_eq!(name, "push");
            assert_eq!((module_path.0).0, "std");
            assert_eq!(module_path.1, vec!["vec", "Vec"]);
        }
        _ => panic!("Expected ModuleItem with Function kind"),
    }
}

#[test]
fn test_rustdoc_type_conversion() {
    use serde_json::json;
    use veltrano::rust_interop::RustdocQuerier;

    // Create comprehensive mock rustdoc JSON structures for testing
    let rustdoc_json = json!({
        "crate_name": "test_crate",
        "crate_version": "0.1.0",
        "paths": {
            "0:0": {
                "crate_id": 0,
                "path": ["test_crate", "geometry", "Point"],
                "kind": "struct"
            },
            "0:1": {
                "crate_id": 0,
                "path": ["test_crate", "Option"],
                "kind": "enum"
            },
            "0:2": {
                "crate_id": 1,
                "path": ["libc", "c_void"],
                "kind": "union"
            }
        },
        "external_crates": {
            "1": {
                "name": "libc",
                "html_root_url": null
            }
        },
        "index": {
            "0:0": {
                "id": "0:0",
                "crate_id": 0,
                "name": "Point",
                "kind": "struct",
                "inner": {
                    "fields": [
                        {
                            "name": "x",
                            "type": "f64",
                            "is_public": true
                        },
                        {
                            "name": "y",
                            "type": "f64",
                            "is_public": true
                        },
                        {
                            "name": "label",
                            "type": "Option<String>",
                            "is_public": false
                        }
                    ],
                    "generics": {
                        "params": []
                    }
                }
            },
            "0:1": {
                "id": "0:1",
                "crate_id": 0,
                "name": "Option",
                "kind": "enum",
                "inner": {
                    "variants": [
                        {
                            "name": "Some",
                            "fields": [
                                {
                                    "name": "0",
                                    "type": "T",
                                    "is_public": true
                                }
                            ]
                        },
                        {
                            "name": "None",
                            "fields": []
                        }
                    ],
                    "generics": {
                        "params": [
                            {
                                "name": "T",
                                "bounds": [],
                                "default": null
                            }
                        ]
                    }
                }
            },
            "0:2": {
                "id": "0:2",
                "crate_id": 1,
                "name": "c_void",
                "kind": "union",
                "inner": {
                    "fields": [
                        {
                            "name": "_private",
                            "type": "[u8; 0]",
                            "is_public": false
                        }
                    ],
                    "generics": {
                        "params": []
                    }
                }
            }
        }
    });

    // Test that we can parse and convert the rustdoc JSON
    let querier = RustdocQuerier::new(None);
    let crate_info = querier
        .convert_rustdoc_to_crate_info(serde_json::from_value(rustdoc_json).unwrap())
        .unwrap();

    // Test struct conversion
    let point_type = crate_info.types.get("Point").unwrap();
    assert_eq!(point_type.name, "Point");
    assert_eq!(point_type.kind, TypeKind::Struct);

    // Verify module path extraction for struct
    match &point_type.path {
        RustPath::Type(type_path) => {
            assert_eq!(((type_path.0).0).0, "test_crate");
            assert_eq!((type_path.0).1, vec!["geometry"]);
            assert_eq!(type_path.1, vec!["Point"]);
        }
        _ => panic!("Expected Type path"),
    }

    // Verify struct fields with parsed types
    assert_eq!(point_type.fields.len(), 3);
    assert_eq!(point_type.fields[0].name, "x");
    assert_eq!(point_type.fields[0].field_type.raw, "f64");
    assert!(matches!(
        point_type.fields[0].field_type.parsed,
        Some(RustType::Custom { .. })
    ));
    assert!(point_type.fields[0].is_public);

    assert_eq!(point_type.fields[2].name, "label");
    assert_eq!(point_type.fields[2].field_type.raw, "Option<String>");
    assert!(matches!(
        point_type.fields[2].field_type.parsed,
        Some(RustType::Option(_))
    ));
    assert!(!point_type.fields[2].is_public);

    // Test enum conversion
    let option_type = crate_info.types.get("Option").unwrap();
    assert_eq!(option_type.name, "Option");
    assert_eq!(option_type.kind, TypeKind::Enum);

    // Verify enum variants
    assert_eq!(option_type.variants.len(), 2);
    assert_eq!(option_type.variants[0].name, "Some");
    assert_eq!(option_type.variants[0].fields.len(), 1);
    assert_eq!(option_type.variants[0].fields[0].field_type.raw, "T");
    assert!(matches!(
        option_type.variants[0].fields[0].field_type.parsed,
        Some(RustType::Generic(_))
    ));

    assert_eq!(option_type.variants[1].name, "None");
    assert_eq!(option_type.variants[1].fields.len(), 0);

    // Test union conversion from external crate
    let c_void_type = crate_info.types.get("c_void").unwrap();
    assert_eq!(c_void_type.name, "c_void");
    assert_eq!(c_void_type.kind, TypeKind::Union);

    // Verify external crate path
    match &c_void_type.path {
        RustPath::Type(type_path) => {
            assert_eq!(((type_path.0).0).0, "libc");
            assert!(((type_path.0).1).is_empty());
            assert_eq!(type_path.1, vec!["c_void"]);
        }
        _ => panic!("Expected Type path"),
    }
}

#[test]
fn test_rustdoc_method_extraction() {
    use serde_json::json;
    use veltrano::rust_interop::RustdocQuerier;

    // Create a comprehensive mock rustdoc JSON structure with methods
    let rustdoc_json = json!({
        "crate_name": "test_crate",
        "crate_version": "0.1.0",
        "paths": {
            "0:0": {
                "crate_id": 0,
                "path": ["test_crate", "MyStruct"],
                "kind": "struct"
            },
            "0:1": {
                "crate_id": 0,
                "path": ["test_crate", "impl"],
                "kind": "impl"
            },
            "0:2": {
                "crate_id": 0,
                "path": ["test_crate", "MyStruct", "new"],
                "kind": "method"
            },
            "0:3": {
                "crate_id": 0,
                "path": ["test_crate", "MyStruct", "get_value"],
                "kind": "method"
            },
            "0:4": {
                "crate_id": 0,
                "path": ["test_crate", "MyStruct", "set_value"],
                "kind": "method"
            }
        },
        "external_crates": {},
        "index": {
            "0:0": {
                "id": "0:0",
                "crate_id": 0,
                "name": "MyStruct",
                "kind": "struct",
                "inner": {
                    "fields": [
                        {
                            "name": "value",
                            "type": "T",
                            "is_public": false
                        }
                    ],
                    "generics": {
                        "params": [
                            {
                                "name": "T",
                                "bounds": ["Clone"],
                                "default": null
                            }
                        ]
                    },
                    "impls": ["0:1"]
                }
            },
            "0:1": {
                "id": "0:1",
                "crate_id": 0,
                "name": "",
                "kind": "impl",
                "inner": {
                    "for": {},
                    "trait": null,
                    "items": ["0:2", "0:3", "0:4"]
                }
            },
            "0:2": {
                "id": "0:2",
                "crate_id": 0,
                "name": "new",
                "kind": "method",
                "inner": {
                    "name": "new",
                    "sig": {
                        "inputs": [
                            ["value", "T"]
                        ],
                        "output": "Self"
                    },
                    "generics": {
                        "params": []
                    },
                    "header": {
                        "is_const": false,
                        "is_unsafe": false,
                        "is_async": false
                    },
                    "has_body": true
                }
            },
            "0:3": {
                "id": "0:3",
                "crate_id": 0,
                "name": "get_value",
                "kind": "method",
                "inner": {
                    "name": "get_value",
                    "sig": {
                        "inputs": [
                            ["&self", "&Self"]
                        ],
                        "output": "&T"
                    },
                    "generics": {
                        "params": []
                    },
                    "header": {
                        "is_const": false,
                        "is_unsafe": false,
                        "is_async": false
                    },
                    "has_body": true
                }
            },
            "0:4": {
                "id": "0:4",
                "crate_id": 0,
                "name": "set_value",
                "kind": "method",
                "inner": {
                    "name": "set_value",
                    "sig": {
                        "inputs": [
                            ["&mut self", "&mut Self"],
                            ["value", "T"]
                        ],
                        "output": null
                    },
                    "generics": {
                        "params": []
                    },
                    "header": {
                        "is_const": false,
                        "is_unsafe": false,
                        "is_async": false
                    },
                    "has_body": true
                }
            }
        }
    });

    // Test that we can parse and convert the rustdoc JSON
    let querier = RustdocQuerier::new(None);
    let crate_info = querier
        .convert_rustdoc_to_crate_info(serde_json::from_value(rustdoc_json).unwrap())
        .unwrap();

    // Get the struct type
    let my_struct = crate_info.types.get("MyStruct").unwrap();
    assert_eq!(my_struct.name, "MyStruct");

    // Verify methods were extracted
    assert_eq!(my_struct.methods.len(), 3);

    // Check static method (no self)
    let new_method = my_struct.methods.iter().find(|m| m.name == "new").unwrap();
    assert_eq!(new_method.name, "new");
    assert!(matches!(new_method.self_kind, SelfKind::None));
    assert_eq!(new_method.parameters.len(), 1);
    assert_eq!(new_method.parameters[0].name, "value");
    assert_eq!(new_method.return_type.raw, "Self");

    // Check immutable self method
    let get_method = my_struct
        .methods
        .iter()
        .find(|m| m.name == "get_value")
        .unwrap();
    assert_eq!(get_method.name, "get_value");
    assert!(matches!(get_method.self_kind, SelfKind::Ref(None)));
    assert_eq!(get_method.parameters.len(), 0); // self is not in parameters
    assert_eq!(get_method.return_type.raw, "&T");

    // Check mutable self method
    let set_method = my_struct
        .methods
        .iter()
        .find(|m| m.name == "set_value")
        .unwrap();
    assert_eq!(set_method.name, "set_value");
    assert!(matches!(set_method.self_kind, SelfKind::MutRef(None)));
    assert_eq!(set_method.parameters.len(), 1);
    assert_eq!(set_method.parameters[0].name, "value");
    assert_eq!(set_method.return_type.raw, "()");
}
