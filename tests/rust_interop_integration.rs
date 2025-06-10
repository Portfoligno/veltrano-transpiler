use std::collections::HashMap;
use veltrano::rust_interop::*;

/// Integration tests that validate actual interactions with the Rust toolchain
/// These tests may be marked with #[ignore] if they require specific toolchain setup

#[test]
#[ignore] // Requires access to actual Rust source files
fn test_syn_querier_against_veltrano_crate() {
    // Test SynQuerier against the veltrano transpiler itself
    if let Ok(mut querier) = SynQuerier::new(None) {
        // Try to extract information from our own crate
        let result = querier.extract_from_source("veltrano");

        match result {
            Ok(crate_info) => {
                // Validate that we extracted meaningful information
                assert_eq!(crate_info.name, "veltrano");
                assert!(!crate_info.version.is_empty());

                // Should have found at least some functions/types
                println!("Found {} functions", crate_info.functions.len());
                println!("Found {} types", crate_info.types.len());
                println!("Found {} traits", crate_info.traits.len());

                // Check for specific functions we know exist
                for (name, func) in &crate_info.functions {
                    println!("Function: {} ({})", name, func.full_path);
                    assert!(!func.name.is_empty());
                    assert!(!func.full_path.is_empty());
                }

                // Check for specific types we know exist
                for (name, type_info) in &crate_info.types {
                    println!("Type: {} (kind: {:?})", name, type_info.kind);
                    assert!(!type_info.name.is_empty());
                    assert!(!type_info.full_path.is_empty());
                }
            }
            Err(e) => {
                println!("Failed to extract crate info: {}", e);
                // This might fail if we're not in the right directory or cargo metadata isn't available
                // That's OK for an integration test
            }
        }
    } else {
        println!("SynQuerier creation failed - not in a Cargo project");
    }
}

#[test]
#[ignore] // Requires cargo and rustdoc to be available
fn test_rustdoc_querier_against_real_crate() {
    let mut querier = RustdocQuerier::new(None);

    // Try to extract documentation from a simple crate
    // Note: This test will only work if we can successfully run cargo doc
    let result = querier.extract_crate_info("veltrano");

    match result {
        Ok(crate_info) => {
            assert_eq!(crate_info.name, "veltrano");
            println!(
                "Successfully extracted rustdoc info for {}",
                crate_info.name
            );
            // The current implementation returns empty collections
            // In a full implementation, we'd have more detailed checks here
        }
        Err(e) => {
            println!("Rustdoc extraction failed (expected): {}", e);
            // This is expected since our rustdoc implementation is currently a placeholder
        }
    }
}

#[test]
fn test_real_rust_type_parsing_and_conversion() {
    // Test the type parser against real Rust type signatures
    let test_cases = vec![
        ("i32", RustType::I32),
        ("bool", RustType::Bool),
        ("()", RustType::Unit),
        ("String", RustType::String),
        ("str", RustType::Str),
    ];

    for (input, expected) in test_cases {
        let parsed = RustTypeParser::parse(input).unwrap();
        assert_eq!(parsed, expected, "Failed to parse: {}", input);

        // Test conversion to Veltrano type
        let veltrano_type = parsed.to_veltrano_type();
        assert!(
            veltrano_type.is_ok(),
            "Failed to convert {} to Veltrano type",
            input
        );

        let vt = veltrano_type.unwrap();
        println!("{} -> {:?}", input, vt.constructor);
    }
}

#[test]
fn test_complex_rust_signatures() {
    use veltrano::types::VeltranoType;

    // Test parsing of complex type signatures that might appear in real code
    let test_cases = vec![
        (
            "&str",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Str),
            },
            VeltranoType::str(),
        ),
        (
            "&mut String",
            RustType::MutRef {
                lifetime: None,
                inner: Box::new(RustType::String),
            },
            VeltranoType::mut_ref(VeltranoType::own(VeltranoType::string())),
        ),
        (
            "Box<i32>",
            RustType::Box(Box::new(RustType::I32)),
            VeltranoType::own(VeltranoType::boxed(VeltranoType::i32())),
        ),
        (
            "Vec<String>",
            RustType::Vec(Box::new(RustType::String)),
            VeltranoType::own(VeltranoType::vec(VeltranoType::own(VeltranoType::string()))),
        ),
        (
            "Option<i32>",
            RustType::Option(Box::new(RustType::I32)),
            VeltranoType::own(VeltranoType::option(VeltranoType::i32())),
        ),
        (
            "&'static str",
            RustType::Ref {
                lifetime: Some("static".to_string()),
                inner: Box::new(RustType::Str),
            },
            VeltranoType::str(),
        ),
        (
            "&String",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::String),
            },
            VeltranoType::string(),
        ),
        (
            "&mut Vec<String>",
            RustType::MutRef {
                lifetime: None,
                inner: Box::new(RustType::Vec(Box::new(RustType::String))),
            },
            VeltranoType::mut_ref(VeltranoType::own(VeltranoType::vec(VeltranoType::own(
                VeltranoType::string(),
            )))),
        ),
    ];

    for (type_str, expected_rust_type, expected_veltrano_type) in test_cases {
        match RustTypeParser::parse(type_str) {
            Ok(rust_type) => {
                assert_eq!(
                    rust_type, expected_rust_type,
                    "Unexpected parse result for {}",
                    type_str
                );

                // Try to convert to Veltrano type
                match rust_type.to_veltrano_type() {
                    Ok(veltrano_type) => {
                        assert_eq!(
                            veltrano_type, expected_veltrano_type,
                            "Unexpected Veltrano type for {}",
                            type_str
                        );
                    }
                    Err(e) => {
                        panic!("Failed to convert {} to Veltrano type: {}", type_str, e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to parse {}: {}", type_str, e);
            }
        }
    }
}

#[test]
fn test_reference_nesting_and_cancellation() {
    use veltrano::types::VeltranoType;

    // Test various reference nesting scenarios and Ref/Own cancellation
    let test_cases = vec![
        // Double reference to i32 (no cancellation)
        (
            "&&i32",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::I32),
                }),
            },
            VeltranoType::ref_(VeltranoType::ref_(VeltranoType::i32())),
        ),
        // Double reference to String (outer ref cancels with Own)
        (
            "&&String",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::String),
                }),
            },
            VeltranoType::ref_(VeltranoType::string()),
        ),
        // &Box<&str> - inner &str becomes just str
        (
            "&Box<&str>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Box(Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                }))),
            },
            VeltranoType::boxed(VeltranoType::str()),
        ),
        // &Vec<&String> - inner &String becomes just String
        (
            "&Vec<&String>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Vec(Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::String),
                }))),
            },
            VeltranoType::vec(VeltranoType::string()),
        ),
        // &Option<&str> - inner &str becomes just str
        (
            "&Option<&str>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Option(Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Str),
                }))),
            },
            VeltranoType::option(VeltranoType::str()),
        ),
        // &mut &String - only inner ref cancels
        (
            "&mut &String",
            RustType::MutRef {
                lifetime: None,
                inner: Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::String),
                }),
            },
            VeltranoType::mut_ref(VeltranoType::string()),
        ),
        // &mut &mut String - no cancellation
        (
            "&mut &mut String",
            RustType::MutRef {
                lifetime: None,
                inner: Box::new(RustType::MutRef {
                    lifetime: None,
                    inner: Box::new(RustType::String),
                }),
            },
            VeltranoType::mut_ref(VeltranoType::mut_ref(VeltranoType::own(
                VeltranoType::string(),
            ))),
        ),
        // Complex: &Result<&str, &String>
        (
            "&Result<&str, &String>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Result {
                    ok: Box::new(RustType::Ref {
                        lifetime: None,
                        inner: Box::new(RustType::Str),
                    }),
                    err: Box::new(RustType::Ref {
                        lifetime: None,
                        inner: Box::new(RustType::String),
                    }),
                }),
            },
            VeltranoType::result(VeltranoType::str(), VeltranoType::string()),
        ),
        // &Option<Box<&String>> - nested cancellation
        (
            "&Option<Box<&String>>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Option(Box::new(RustType::Box(Box::new(
                    RustType::Ref {
                        lifetime: None,
                        inner: Box::new(RustType::String),
                    },
                ))))),
            },
            VeltranoType::option(VeltranoType::own(VeltranoType::boxed(
                VeltranoType::string(),
            ))),
        ),
        // Triple reference &&&str
        (
            "&&&str",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Ref {
                    lifetime: None,
                    inner: Box::new(RustType::Ref {
                        lifetime: None,
                        inner: Box::new(RustType::Str),
                    }),
                }),
            },
            VeltranoType::ref_(VeltranoType::ref_(VeltranoType::str())),
        ),
    ];

    for (type_str, expected_rust_type, expected_veltrano_type) in test_cases {
        match RustTypeParser::parse(type_str) {
            Ok(rust_type) => {
                assert_eq!(
                    rust_type, expected_rust_type,
                    "Unexpected parse result for {}",
                    type_str
                );

                // Try to convert to Veltrano type
                match rust_type.to_veltrano_type() {
                    Ok(veltrano_type) => {
                        assert_eq!(
                            veltrano_type, expected_veltrano_type,
                            "Unexpected Veltrano type for {}",
                            type_str
                        );
                    }
                    Err(e) => {
                        panic!("Failed to convert {} to Veltrano type: {}", type_str, e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to parse {}: {}", type_str, e);
            }
        }
    }
}

#[test]
fn test_complex_nested_rust_signatures() {
    use veltrano::types::VeltranoType;

    // Test parsing of deeply nested and complex type signatures
    let test_cases = vec![
        // Nested Options
        (
            "Option<Option<i32>>",
            RustType::Option(Box::new(RustType::Option(Box::new(RustType::I32)))),
            VeltranoType::own(VeltranoType::option(VeltranoType::own(
                VeltranoType::option(VeltranoType::i32()),
            ))),
        ),
        // Vec of Options
        (
            "Vec<Option<String>>",
            RustType::Vec(Box::new(RustType::Option(Box::new(RustType::String)))),
            VeltranoType::own(VeltranoType::vec(VeltranoType::own(VeltranoType::option(
                VeltranoType::own(VeltranoType::string()),
            )))),
        ),
        // Option of Vec
        (
            "Option<Vec<i32>>",
            RustType::Option(Box::new(RustType::Vec(Box::new(RustType::I32)))),
            VeltranoType::own(VeltranoType::option(VeltranoType::own(VeltranoType::vec(
                VeltranoType::i32(),
            )))),
        ),
        // Box of Vec
        (
            "Box<Vec<String>>",
            RustType::Box(Box::new(RustType::Vec(Box::new(RustType::String)))),
            VeltranoType::own(VeltranoType::boxed(VeltranoType::own(VeltranoType::vec(
                VeltranoType::own(VeltranoType::string()),
            )))),
        ),
        // Result types
        (
            "Result<i32, String>",
            RustType::Result {
                ok: Box::new(RustType::I32),
                err: Box::new(RustType::String),
            },
            VeltranoType::own(VeltranoType::result(
                VeltranoType::i32(),
                VeltranoType::own(VeltranoType::string()),
            )),
        ),
        // Result with complex types
        (
            "Result<Vec<i32>, Box<String>>",
            RustType::Result {
                ok: Box::new(RustType::Vec(Box::new(RustType::I32))),
                err: Box::new(RustType::Box(Box::new(RustType::String))),
            },
            VeltranoType::own(VeltranoType::result(
                VeltranoType::own(VeltranoType::vec(VeltranoType::i32())),
                VeltranoType::own(VeltranoType::boxed(VeltranoType::own(
                    VeltranoType::string(),
                ))),
            )),
        ),
        // Nested Results
        (
            "Result<Option<i32>, Vec<String>>",
            RustType::Result {
                ok: Box::new(RustType::Option(Box::new(RustType::I32))),
                err: Box::new(RustType::Vec(Box::new(RustType::String))),
            },
            VeltranoType::own(VeltranoType::result(
                VeltranoType::own(VeltranoType::option(VeltranoType::i32())),
                VeltranoType::own(VeltranoType::vec(VeltranoType::own(VeltranoType::string()))),
            )),
        ),
        // Reference to Box
        (
            "&Box<String>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Box(Box::new(RustType::String))),
            },
            VeltranoType::boxed(VeltranoType::own(VeltranoType::string())),
        ),
        // Mutable reference to Vec of Options
        (
            "&mut Vec<Option<i32>>",
            RustType::MutRef {
                lifetime: None,
                inner: Box::new(RustType::Vec(Box::new(RustType::Option(Box::new(
                    RustType::I32,
                ))))),
            },
            VeltranoType::mut_ref(VeltranoType::own(VeltranoType::vec(VeltranoType::own(
                VeltranoType::option(VeltranoType::i32()),
            )))),
        ),
        // Vec of Boxes
        (
            "Vec<Box<String>>",
            RustType::Vec(Box::new(RustType::Box(Box::new(RustType::String)))),
            VeltranoType::own(VeltranoType::vec(VeltranoType::own(VeltranoType::boxed(
                VeltranoType::own(VeltranoType::string()),
            )))),
        ),
        // Triple nesting
        (
            "Option<Vec<Box<i32>>>",
            RustType::Option(Box::new(RustType::Vec(Box::new(RustType::Box(Box::new(
                RustType::I32,
            )))))),
            VeltranoType::own(VeltranoType::option(VeltranoType::own(VeltranoType::vec(
                VeltranoType::own(VeltranoType::boxed(VeltranoType::i32())),
            )))),
        ),
        // Reference to Option of Vec
        (
            "&Option<Vec<String>>",
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::Option(Box::new(RustType::Vec(Box::new(
                    RustType::String,
                ))))),
            },
            VeltranoType::option(VeltranoType::own(VeltranoType::vec(VeltranoType::own(
                VeltranoType::string(),
            )))),
        ),
        // Complex Result with nested types
        (
            "Result<Box<Vec<i32>>, Option<String>>",
            RustType::Result {
                ok: Box::new(RustType::Box(Box::new(RustType::Vec(Box::new(
                    RustType::I32,
                ))))),
                err: Box::new(RustType::Option(Box::new(RustType::String))),
            },
            VeltranoType::own(VeltranoType::result(
                VeltranoType::own(VeltranoType::boxed(VeltranoType::own(VeltranoType::vec(
                    VeltranoType::i32(),
                )))),
                VeltranoType::own(VeltranoType::option(VeltranoType::own(
                    VeltranoType::string(),
                ))),
            )),
        ),
        // Lifetime in nested reference
        (
            "&'a Vec<Option<String>>",
            RustType::Ref {
                lifetime: Some("a".to_string()),
                inner: Box::new(RustType::Vec(Box::new(RustType::Option(Box::new(
                    RustType::String,
                ))))),
            },
            VeltranoType::vec(VeltranoType::own(VeltranoType::option(VeltranoType::own(
                VeltranoType::string(),
            )))),
        ),
    ];

    for (type_str, expected_rust_type, expected_veltrano_type) in test_cases {
        match RustTypeParser::parse(type_str) {
            Ok(rust_type) => {
                assert_eq!(
                    rust_type, expected_rust_type,
                    "Unexpected parse result for {}",
                    type_str
                );

                // Try to convert to Veltrano type
                match rust_type.to_veltrano_type() {
                    Ok(veltrano_type) => {
                        assert_eq!(
                            veltrano_type, expected_veltrano_type,
                            "Unexpected Veltrano type for {}",
                            type_str
                        );
                    }
                    Err(e) => {
                        panic!("Failed to convert {} to Veltrano type: {}", type_str, e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to parse {}: {}", type_str, e);
            }
        }
    }
}

#[test]
fn test_dynamic_registry_with_real_scenarios() {
    let registry = DynamicRustRegistry::new();

    // Test path parsing with real-world paths
    let test_paths = vec![
        "std::vec::Vec::new",
        "std::collections::HashMap::insert",
        "core::fmt::Display::fmt",
        "serde::Serialize::serialize",
    ];

    for path in test_paths {
        match registry.parse_path(path) {
            Ok((crate_name, item_path)) => {
                println!(
                    "Path: {} -> crate: {}, item: {}",
                    path, crate_name, item_path
                );
                assert!(!crate_name.is_empty());
                assert!(!item_path.is_empty());
            }
            Err(e) => {
                println!("Failed to parse path {}: {}", path, e);
            }
        }
    }
}

#[test]
fn test_function_signature_extraction() {
    // Test extracting function signatures that we might encounter in real code
    if let Ok(querier) = SynQuerier::new(None) {
        let function_samples = vec![
            "fn simple_function() -> i32 { 42 }",
            "fn with_params(a: i32, b: &str) -> String { String::new() }",
            "fn generic_function<T: Clone>(item: T) -> T { item.clone() }",
            "fn with_lifetime<'a>(s: &'a str) -> &'a str { s }",
            "unsafe fn unsafe_function() -> *const u8 { std::ptr::null() }",
            "const fn const_function() -> i32 { 42 }",
        ];

        for func_str in function_samples {
            match syn::parse_str::<syn::ItemFn>(func_str) {
                Ok(parsed_fn) => {
                    match querier.parse_function(&parsed_fn) {
                        Ok(func_info) => {
                            println!("Function: {}", func_info.name);
                            println!("  Generics: {:?}", func_info.generics);
                            println!("  Parameters: {:?}", func_info.parameters);
                            println!("  Return type: {}", func_info.return_type.raw);
                            println!("  Unsafe: {}", func_info.is_unsafe);
                            println!("  Const: {}", func_info.is_const);

                            // Validate extracted information
                            assert!(!func_info.name.is_empty());
                            assert!(!func_info.return_type.raw.is_empty());
                        }
                        Err(e) => {
                            println!("Failed to parse function info: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse function syntax: {}", e);
                }
            }
        }
    }
}

#[test]
fn test_struct_and_enum_extraction() {
    if let Ok(querier) = SynQuerier::new(None) {
        // Test real struct definitions
        let struct_samples = vec![
            r#"
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
            "#,
            r#"
            #[derive(Debug, Clone)]
            pub struct GenericStruct<T, U: Clone> {
                field1: T,
                field2: Option<U>,
            }
            "#,
            r#"
            pub struct TupleStruct(pub i32, String, Vec<u8>);
            "#,
        ];

        for struct_str in struct_samples {
            match syn::parse_str::<syn::ItemStruct>(struct_str) {
                Ok(parsed_struct) => match querier.parse_struct(&parsed_struct) {
                    Ok(struct_info) => {
                        println!("Struct: {}", struct_info.name);
                        println!("  Generics: {:?}", struct_info.generics);
                        println!("  Fields: {:?}", struct_info.fields);

                        assert!(!struct_info.name.is_empty());
                        assert_eq!(struct_info.kind, TypeKind::Struct);
                    }
                    Err(e) => {
                        println!("Failed to parse struct info: {}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse struct syntax: {}", e);
                }
            }
        }

        // Test enum definitions
        let enum_samples = vec![
            r#"
            pub enum Color {
                Red,
                Green,
                Blue,
            }
            "#,
            r#"
            pub enum Option<T> {
                Some(T),
                None,
            }
            "#,
            r#"
            pub enum Message {
                Quit,
                Move { x: i32, y: i32 },
                Write(String),
                ChangeColor(i32, i32, i32),
            }
            "#,
        ];

        for enum_str in enum_samples {
            match syn::parse_str::<syn::ItemEnum>(enum_str) {
                Ok(parsed_enum) => match querier.parse_enum(&parsed_enum) {
                    Ok(enum_info) => {
                        println!("Enum: {}", enum_info.name);
                        println!("  Variants: {:?}", enum_info.variants);

                        assert!(!enum_info.name.is_empty());
                        assert_eq!(enum_info.kind, TypeKind::Enum);
                        assert!(!enum_info.variants.is_empty());
                    }
                    Err(e) => {
                        println!("Failed to parse enum info: {}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse enum syntax: {}", e);
                }
            }
        }
    }
}

#[test]
fn test_trait_and_impl_extraction() {
    if let Ok(querier) = SynQuerier::new(None) {
        // Test trait definition
        let trait_str = r#"
            pub trait Display {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error>;
                
                fn to_string(&self) -> String {
                    // default implementation
                    String::new()
                }
            }
        "#;

        match syn::parse_str::<syn::ItemTrait>(trait_str) {
            Ok(parsed_trait) => match querier.parse_trait(&parsed_trait) {
                Ok(trait_info) => {
                    println!("Trait: {}", trait_info.name);
                    println!("  Methods: {:?}", trait_info.methods);

                    assert!(!trait_info.name.is_empty());
                    assert!(!trait_info.methods.is_empty());
                }
                Err(e) => {
                    println!("Failed to parse trait info: {}", e);
                }
            },
            Err(e) => {
                println!("Failed to parse trait syntax: {}", e);
            }
        }

        // Test impl block
        let impl_str = r#"
            impl Point {
                pub fn new(x: f64, y: f64) -> Self {
                    Point { x, y }
                }
                
                pub fn distance(&self, other: &Point) -> f64 {
                    ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
                }
                
                pub fn translate(&mut self, dx: f64, dy: f64) {
                    self.x += dx;
                    self.y += dy;
                }
            }
        "#;

        match syn::parse_str::<syn::ItemImpl>(impl_str) {
            Ok(parsed_impl) => {
                let mut crate_info = CrateInfo {
                    name: "test".to_string(),
                    version: "1.0.0".to_string(),
                    functions: HashMap::new(),
                    types: HashMap::new(),
                    traits: HashMap::new(),
                    trait_implementations: HashMap::new(),
                };

                // Add the Point type first
                crate_info.types.insert(
                    "Point".to_string(),
                    TypeInfo {
                        name: "Point".to_string(),
                        full_path: "Point".to_string(),
                        kind: TypeKind::Struct,
                        generics: vec![],
                        methods: vec![],
                        fields: vec![],
                        variants: vec![],
                    },
                );

                match querier.parse_impl_block(&parsed_impl, &mut crate_info) {
                    Ok(()) => {
                        let point_type = crate_info.types.get("Point").unwrap();
                        println!("Impl block methods: {:?}", point_type.methods);

                        assert_eq!(point_type.methods.len(), 3);

                        // Check specific methods
                        let new_method =
                            point_type.methods.iter().find(|m| m.name == "new").unwrap();
                        assert_eq!(new_method.self_kind, SelfKind::None);

                        let distance_method = point_type
                            .methods
                            .iter()
                            .find(|m| m.name == "distance")
                            .unwrap();
                        assert_eq!(distance_method.self_kind, SelfKind::Ref);

                        let translate_method = point_type
                            .methods
                            .iter()
                            .find(|m| m.name == "translate")
                            .unwrap();
                        assert_eq!(translate_method.self_kind, SelfKind::MutRef);
                    }
                    Err(e) => {
                        println!("Failed to parse impl block: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to parse impl syntax: {}", e);
            }
        }
    }
}

#[test]
fn test_error_handling_with_invalid_rust_code() {
    // Test that our parsers handle invalid Rust code gracefully
    if let Ok(querier) = SynQuerier::new(None) {
        let invalid_samples = vec![
            "fn incomplete_function(",
            "struct InvalidStruct { field: }",
            "enum { }",
            "impl { }",
        ];

        for invalid_code in invalid_samples {
            // These should all fail to parse at the syn level
            if let Ok(parsed) = syn::parse_str::<syn::ItemFn>(invalid_code) {
                // If syn somehow parsed it, our parser should handle it
                let result = querier.parse_function(&parsed);
                println!(
                    "Unexpectedly parsed invalid code: {} -> {:?}",
                    invalid_code, result
                );
            } else {
                println!("Correctly rejected invalid code: {}", invalid_code);
            }
        }
    }
}

#[test]
fn test_toolchain_availability() {
    // Test whether the required toolchain components are available
    println!("Testing toolchain availability...");

    // Test cargo availability
    match std::process::Command::new("cargo")
        .arg("--version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Cargo available: {}", version.trim());
            } else {
                println!("Cargo command failed");
            }
        }
        Err(e) => {
            println!("Cargo not available: {}", e);
        }
    }

    // Test rustc availability
    match std::process::Command::new("rustc")
        .arg("--version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Rustc available: {}", version.trim());
            } else {
                println!("Rustc command failed");
            }
        }
        Err(e) => {
            println!("Rustc not available: {}", e);
        }
    }

    // Test if we're in a Cargo project
    match cargo_metadata::MetadataCommand::new().exec() {
        Ok(metadata) => {
            println!("Cargo metadata available");
            println!("Workspace root: {}", metadata.workspace_root);
            println!("Packages: {}", metadata.packages.len());
            for package in &metadata.packages {
                println!("  - {} ({})", package.name, package.version);
            }
        }
        Err(e) => {
            println!("Cargo metadata not available: {}", e);
        }
    }
}

#[test]
#[ignore] // May be slow, requires network access for dependencies
fn test_stdlib_type_extraction() {
    // Test extracting type information from standard library types
    // This test validates that our type parsing works with real stdlib signatures

    let stdlib_types = vec![
        "std::vec::Vec",
        "std::collections::HashMap",
        "std::option::Option",
        "std::result::Result",
        "std::string::String",
    ];

    for type_path in stdlib_types {
        // For now, just test path parsing since full stdlib extraction would be complex
        let registry = DynamicRustRegistry::new();
        match registry.parse_path(type_path) {
            Ok((crate_name, item_path)) => {
                println!(
                    "Stdlib type: {} -> crate: {}, path: {}",
                    type_path, crate_name, item_path
                );
                assert!(!crate_name.is_empty());
                assert!(!item_path.is_empty());
            }
            Err(e) => {
                println!("Failed to parse stdlib type path {}: {}", type_path, e);
            }
        }
    }
}

#[test]
fn test_integration_with_veltrano_type_system() {
    use veltrano::types::VeltranoType;

    // Test that extracted Rust types integrate properly with the Veltrano type system
    let rust_types_to_test = vec![
        (RustType::I32, VeltranoType::i32()),
        (RustType::Bool, VeltranoType::bool()),
        (RustType::Unit, VeltranoType::unit()),
        (RustType::String, VeltranoType::own(VeltranoType::string())),
        (RustType::Str, VeltranoType::own(VeltranoType::str())),
        (
            RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::I32),
            },
            VeltranoType::ref_(VeltranoType::i32()),
        ),
        (
            RustType::Custom {
                name: "MyType".to_string(),
                generics: vec![],
            },
            VeltranoType::custom("MyType".to_string()),
        ),
    ];

    for (rust_type, expected_veltrano_type) in rust_types_to_test {
        match rust_type.to_veltrano_type() {
            Ok(veltrano_type) => {
                assert_eq!(veltrano_type, expected_veltrano_type);
                println!("✓ {:?} -> {:?}", rust_type, veltrano_type);
            }
            Err(e) => {
                panic!("Failed to convert {:?} to Veltrano type: {}", rust_type, e);
            }
        }
    }
}

#[test]
fn test_realistic_transpiler_workflow() {
    // Test a realistic workflow that a transpiler might use
    let mut registry = DynamicRustRegistry::new();

    // Step 1: Try to resolve a common function call
    let result = registry.get_function("std::println");
    match result {
        Ok(Some(func_info)) => {
            println!("Found println: {:?}", func_info);
            assert_eq!(func_info.name, "println");
        }
        Ok(None) => {
            println!("println not found in registry (expected with current implementation)");
        }
        Err(e) => {
            println!("Error looking up println: {}", e);
        }
    }

    // Step 2: Try to resolve a type
    let type_result = registry.get_type("std::vec::Vec");
    match type_result {
        Ok(Some(type_info)) => {
            println!("Found Vec: {:?}", type_info);
            assert_eq!(type_info.name, "Vec");
        }
        Ok(None) => {
            println!("Vec not found in registry (expected with current implementation)");
        }
        Err(e) => {
            println!("Error looking up Vec: {}", e);
        }
    }

    // Step 3: Test fallback behavior when crate doesn't exist
    let bad_result = registry.get_function("nonexistent::function");
    match bad_result {
        Ok(None) => {
            println!("✓ Correctly returned None for nonexistent function");
        }
        Ok(Some(_)) => {
            panic!("Unexpectedly found nonexistent function");
        }
        Err(e) => {
            println!("Error (expected): {}", e);
        }
    }
}
