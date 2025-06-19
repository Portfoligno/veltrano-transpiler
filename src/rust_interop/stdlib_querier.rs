//! Standard library type information provider.
//!
//! Provides hardcoded knowledge of common std types and traits.

use super::cache::{
    CrateInfo, GenericParam, MethodInfo, Parameter, RustTypeSignature, TraitInfo, TypeInfo,
    TypeKind,
};
use super::types::{RustType, SelfKind};
use super::{RustInteropError, RustQuerier};
use crate::error::VeltranoError;
use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

/// Standard library querier with minimal hardcoded knowledge
/// This will be replaced with proper rustdoc parsing in the future
#[derive(Debug)]
pub struct StdLibQuerier {
    cache: OnceCell<CrateInfo>,
}

impl StdLibQuerier {
    pub fn new() -> Self {
        Self {
            cache: OnceCell::new(),
        }
    }

    fn create_std_crate_info() -> CrateInfo {
        let mut crate_info = CrateInfo {
            name: "std".to_string(),
            version: "1.0.0".to_string(),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        };

        // Add Clone trait
        let clone_trait = TraitInfo {
            name: "Clone".to_string(),
            full_path: "std::clone::Clone".to_string(),
            methods: vec![MethodInfo {
                name: "clone".to_string(),
                self_kind: SelfKind::Ref,
                generics: vec![],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "Self".to_string(),
                    parsed: None, // Self type
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            associated_types: vec![],
        };
        // Store with just "Clone" since that's what will be looked up after parse_path("std::Clone")
        crate_info.traits.insert("Clone".to_string(), clone_trait);

        // Add ToString trait
        let to_string_trait = TraitInfo {
            name: "ToString".to_string(),
            full_path: "std::string::ToString".to_string(),
            methods: vec![MethodInfo {
                name: "to_string".to_string(),
                self_kind: SelfKind::Ref,
                generics: vec![],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "String".to_string(),
                    parsed: Some(RustType::String),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            associated_types: vec![],
        };
        // Store with just "ToString" since that's what will be looked up after parse_path("std::ToString")
        crate_info
            .traits
            .insert("ToString".to_string(), to_string_trait);

        // Add Into trait
        let into_trait = TraitInfo {
            name: "Into".to_string(),
            full_path: "std::convert::Into".to_string(),
            methods: vec![MethodInfo {
                name: "into".to_string(),
                self_kind: SelfKind::Value,
                generics: vec![GenericParam {
                    name: "T".to_string(),
                    bounds: vec![],
                    default: None,
                }],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "T".to_string(),
                    parsed: Some(RustType::Generic("T".to_string())),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            associated_types: vec![],
        };
        crate_info.traits.insert("Into".to_string(), into_trait);

        // Add trait implementations for common types
        for typ in &[
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64", "bool", "char",
        ] {
            let mut traits = HashSet::new();
            traits.insert("Clone".to_string());
            traits.insert("ToString".to_string());
            crate_info
                .trait_implementations
                .insert(typ.to_string(), traits);
        }

        // String implements Clone, ToString, and Into
        let mut string_traits = HashSet::new();
        string_traits.insert("Clone".to_string());
        string_traits.insert("ToString".to_string());
        string_traits.insert("Into".to_string());
        crate_info
            .trait_implementations
            .insert("String".to_string(), string_traits);

        // &str implements ToString and Into
        let mut str_traits = HashSet::new();
        str_traits.insert("ToString".to_string());
        str_traits.insert("Into".to_string());
        crate_info
            .trait_implementations
            .insert("&str".to_string(), str_traits);

        // Add Vec type with methods
        let vec_type = TypeInfo {
            name: "Vec".to_string(),
            full_path: "std::vec::Vec".to_string(),
            kind: TypeKind::Struct,
            generics: vec![GenericParam {
                name: "T".to_string(),
                bounds: vec![],
                default: None,
            }],
            methods: vec![
                MethodInfo {
                    name: "new".to_string(),
                    self_kind: SelfKind::None, // Static method
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "Vec<T>".to_string(),
                        parsed: Some(RustType::Vec(Box::new(RustType::Generic("T".to_string())))),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "push".to_string(),
                    self_kind: SelfKind::MutRef,
                    generics: vec![],
                    parameters: vec![Parameter {
                        name: "value".to_string(),
                        param_type: RustTypeSignature {
                            raw: "T".to_string(),
                            parsed: Some(RustType::Generic("T".to_string())),
                            lifetimes: vec![],
                            bounds: vec![],
                        },
                    }],
                    return_type: RustTypeSignature {
                        raw: "()".to_string(),
                        parsed: Some(RustType::Unit),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "len".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "usize".to_string(),
                        parsed: Some(RustType::USize),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
            ],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("Vec".to_string(), vec_type);

        // Vec implements Clone
        let mut vec_traits = HashSet::new();
        vec_traits.insert("Clone".to_string());
        crate_info
            .trait_implementations
            .insert("Vec".to_string(), vec_traits);

        // Add numeric type methods (i64 as example)
        let i64_type = TypeInfo {
            name: "i64".to_string(),
            full_path: "i64".to_string(),
            kind: TypeKind::Struct, // Primitive types are treated as structs
            generics: vec![],
            methods: vec![MethodInfo {
                name: "abs".to_string(),
                self_kind: SelfKind::Value, // Takes self by value
                generics: vec![],
                parameters: vec![],
                return_type: RustTypeSignature {
                    raw: "i64".to_string(),
                    parsed: Some(RustType::I64),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_unsafe: false,
                is_const: false,
            }],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("i64".to_string(), i64_type);

        // Add String methods
        let string_type = TypeInfo {
            name: "String".to_string(),
            full_path: "std::string::String".to_string(),
            kind: TypeKind::Struct,
            generics: vec![],
            methods: vec![
                MethodInfo {
                    name: "len".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "usize".to_string(),
                        parsed: Some(RustType::USize),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "chars".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "Chars".to_string(),
                        parsed: Some(RustType::Custom {
                            name: "Chars".to_string(),
                            generics: vec![],
                        }),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "clone".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "String".to_string(),
                        parsed: Some(RustType::String),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
            ],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("String".to_string(), string_type);

        // Add str type with methods
        let str_type = TypeInfo {
            name: "str".to_string(),
            full_path: "str".to_string(),
            kind: TypeKind::Struct, // Primitive types are treated as structs
            generics: vec![],
            methods: vec![
                MethodInfo {
                    name: "len".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "usize".to_string(),
                        parsed: Some(RustType::USize),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
                MethodInfo {
                    name: "to_uppercase".to_string(),
                    self_kind: SelfKind::Ref,
                    generics: vec![],
                    parameters: vec![],
                    return_type: RustTypeSignature {
                        raw: "String".to_string(),
                        parsed: Some(RustType::String),
                        lifetimes: vec![],
                        bounds: vec![],
                    },
                    is_unsafe: false,
                    is_const: false,
                },
            ],
            fields: vec![],
            variants: vec![],
        };
        crate_info.types.insert("str".to_string(), str_type);

        crate_info
    }
}

impl RustQuerier for StdLibQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, VeltranoError> {
        if crate_name == "std" {
            // OnceCell ensures initialization happens only once
            let crate_info = self.cache.get_or_init(Self::create_std_crate_info);
            Ok(crate_info.clone())
        } else {
            Err(VeltranoError::from(RustInteropError::CrateNotFound(
                crate_name.to_string(),
            )))
        }
    }

    fn supports_crate(&self, crate_name: &str) -> bool {
        crate_name == "std"
    }

    fn priority(&self) -> u32 {
        200 // Higher priority than rustdoc and syn queriers
    }
}
