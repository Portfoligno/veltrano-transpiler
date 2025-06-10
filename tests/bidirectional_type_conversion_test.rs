use veltrano::rust_interop::{RustInteropRegistry, RustType, RustTypeParser};
use veltrano::types::VeltranoType;

#[test]
fn test_rust_to_veltrano_to_rust_primitives() {
    let mut registry = RustInteropRegistry::new();

    // Test primitive types roundtrip
    let primitives = vec![
        ("i32", RustType::I32, VeltranoType::i32()),
        ("bool", RustType::Bool, VeltranoType::bool()),
        ("char", RustType::Char, VeltranoType::char()),
        ("()", RustType::Unit, VeltranoType::unit()),
    ];

    for (type_str, expected_rust, expected_veltrano) in primitives {
        // Parse Rust type string
        let rust_type = RustTypeParser::parse(type_str).unwrap();
        assert_eq!(rust_type, expected_rust);

        // Convert to Veltrano
        let veltrano_type = rust_type.to_veltrano_type().unwrap();
        assert_eq!(veltrano_type, expected_veltrano);

        // Convert back to Rust
        let rust_type_back = veltrano_type.to_rust_type(&mut registry);
        assert_eq!(rust_type_back, expected_rust);
    }
}

#[test]
fn test_string_types_with_ownership() {
    let mut registry = RustInteropRegistry::new();

    // Test String type
    let rust_string = RustType::String;
    let veltrano_string = rust_string.to_veltrano_type().unwrap();
    assert_eq!(veltrano_string, VeltranoType::own(VeltranoType::string()));

    // Converting back should give us String (Own removes the reference)
    let rust_back = veltrano_string.to_rust_type(&mut registry);
    assert_eq!(rust_back, RustType::String);

    // Test str type
    let rust_str = RustType::Str;
    let veltrano_str = rust_str.to_veltrano_type().unwrap();
    assert_eq!(veltrano_str, VeltranoType::own(VeltranoType::str()));

    // Converting back should give us str (Own removes the reference)
    let rust_back = veltrano_str.to_rust_type(&mut registry);
    assert_eq!(rust_back, RustType::Str);
}

#[test]
fn test_ref_own_cancellation() {
    let mut registry = RustInteropRegistry::new();

    // Test &String cancels to just String in Veltrano
    let rust_ref_string = RustType::Ref {
        lifetime: None,
        inner: Box::new(RustType::String),
    };
    let veltrano = rust_ref_string.to_veltrano_type().unwrap();
    assert_eq!(veltrano, VeltranoType::string());

    // Converting back should give us &String
    let rust_back = veltrano.to_rust_type(&mut registry);
    assert_eq!(
        rust_back,
        RustType::Ref {
            lifetime: None,
            inner: Box::new(RustType::String),
        }
    );

    // Test &str cancels to just str in Veltrano
    let rust_ref_str = RustType::Ref {
        lifetime: None,
        inner: Box::new(RustType::Str),
    };
    let veltrano = rust_ref_str.to_veltrano_type().unwrap();
    assert_eq!(veltrano, VeltranoType::str());

    // Converting back should give us &str
    let rust_back = veltrano.to_rust_type(&mut registry);
    assert_eq!(
        rust_back,
        RustType::Ref {
            lifetime: None,
            inner: Box::new(RustType::Str),
        }
    );
}

#[test]
fn test_container_types_with_own() {
    let mut registry = RustInteropRegistry::new();

    // Test Vec<i32>
    let rust_vec = RustType::Vec(Box::new(RustType::I32));
    let veltrano_vec = rust_vec.to_veltrano_type().unwrap();
    assert_eq!(
        veltrano_vec,
        VeltranoType::own(VeltranoType::vec(VeltranoType::i32()))
    );

    // Converting back should give us Vec<i32>
    let rust_back = veltrano_vec.to_rust_type(&mut registry);
    assert_eq!(rust_back, RustType::Vec(Box::new(RustType::I32)));

    // Test Option<String>
    let rust_option = RustType::Option(Box::new(RustType::String));
    let veltrano_option = rust_option.to_veltrano_type().unwrap();
    assert_eq!(
        veltrano_option,
        VeltranoType::own(VeltranoType::option(VeltranoType::own(
            VeltranoType::string()
        )))
    );

    // Converting back - inner String stays as String (Own<String> removes the reference)
    let rust_back = veltrano_option.to_rust_type(&mut registry);
    assert_eq!(rust_back, RustType::Option(Box::new(RustType::String)));

    // Test Box<i32>
    let rust_box = RustType::Box(Box::new(RustType::I32));
    let veltrano_box = rust_box.to_veltrano_type().unwrap();
    assert_eq!(
        veltrano_box,
        VeltranoType::own(VeltranoType::boxed(VeltranoType::i32()))
    );

    // Converting back should give us Box<i32>
    let rust_back = veltrano_box.to_rust_type(&mut registry);
    assert_eq!(rust_back, RustType::Box(Box::new(RustType::I32)));
}

#[test]
fn test_nested_ref_own_cancellation() {
    // Test &Vec<String>
    let rust_type = RustType::Ref {
        lifetime: None,
        inner: Box::new(RustType::Vec(Box::new(RustType::String))),
    };
    let veltrano = rust_type.to_veltrano_type().unwrap();
    // Ref cancels with Own around Vec
    assert_eq!(
        veltrano,
        VeltranoType::vec(VeltranoType::own(VeltranoType::string()))
    );

    // Test &Box<String>
    let rust_type = RustType::Ref {
        lifetime: None,
        inner: Box::new(RustType::Box(Box::new(RustType::String))),
    };
    let veltrano = rust_type.to_veltrano_type().unwrap();
    // Ref cancels with Own around Box
    assert_eq!(
        veltrano,
        VeltranoType::boxed(VeltranoType::own(VeltranoType::string()))
    );
}

#[test]
fn test_veltrano_own_to_rust() {
    let mut registry = RustInteropRegistry::new();

    // Test Own<String> removes the reference
    let veltrano = VeltranoType::own(VeltranoType::string());
    let rust = veltrano.to_rust_type(&mut registry);
    assert_eq!(rust, RustType::String);

    // Test Own<Str> removes the reference
    let veltrano = VeltranoType::own(VeltranoType::str());
    let rust = veltrano.to_rust_type(&mut registry);
    assert_eq!(rust, RustType::Str);

    // Test Own<Vec<i32>> removes the reference
    let veltrano = VeltranoType::own(VeltranoType::vec(VeltranoType::i32()));
    let rust = veltrano.to_rust_type(&mut registry);
    assert_eq!(rust, RustType::Vec(Box::new(RustType::I32)));
}

#[test]
fn test_veltrano_ref_to_rust() {
    let mut registry = RustInteropRegistry::new();

    // Test Ref<i32>
    let veltrano = VeltranoType::ref_(VeltranoType::i32());
    let rust = veltrano.to_rust_type(&mut registry);
    assert_eq!(
        rust,
        RustType::Ref {
            lifetime: None,
            inner: Box::new(RustType::I32),
        }
    );

    // Test Ref<String> - inner String becomes &String
    let veltrano = VeltranoType::ref_(VeltranoType::string());
    let rust = veltrano.to_rust_type(&mut registry);
    assert_eq!(
        rust,
        RustType::Ref {
            lifetime: None,
            inner: Box::new(RustType::Ref {
                lifetime: None,
                inner: Box::new(RustType::String),
            }),
        }
    );
}

#[test]
fn test_complex_nested_conversions() {
    let mut registry = RustInteropRegistry::new();

    // Test Result<Vec<i32>, String>
    let rust_result = RustType::Result {
        ok: Box::new(RustType::Vec(Box::new(RustType::I32))),
        err: Box::new(RustType::String),
    };
    let veltrano = rust_result.to_veltrano_type().unwrap();
    assert_eq!(
        veltrano,
        VeltranoType::own(VeltranoType::result(
            VeltranoType::own(VeltranoType::vec(VeltranoType::i32())),
            VeltranoType::own(VeltranoType::string())
        ))
    );

    // Convert back - String stays as String (Own<String> removes the reference)
    let rust_back = veltrano.to_rust_type(&mut registry);
    assert_eq!(
        rust_back,
        RustType::Result {
            ok: Box::new(RustType::Vec(Box::new(RustType::I32))),
            err: Box::new(RustType::String),
        }
    );
}

#[test]
fn test_mutref_conversions() {
    let mut registry = RustInteropRegistry::new();

    // Test &mut Vec<String>
    let rust_mutref = RustType::MutRef {
        lifetime: None,
        inner: Box::new(RustType::Vec(Box::new(RustType::String))),
    };
    let veltrano = rust_mutref.to_veltrano_type().unwrap();
    assert_eq!(
        veltrano,
        VeltranoType::mut_ref(VeltranoType::own(VeltranoType::vec(VeltranoType::own(
            VeltranoType::string()
        ))))
    );

    // Convert back - inner String stays as String (Own<String> removes the reference)
    let rust_back = veltrano.to_rust_type(&mut registry);
    assert_eq!(
        rust_back,
        RustType::MutRef {
            lifetime: None,
            inner: Box::new(RustType::Vec(Box::new(RustType::String))),
        }
    );
}
