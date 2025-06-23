//! Rust type definitions and conversions

use crate::types::{TypeConstructor, VeltranoType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustType {
    // Primitive types
    I32,
    I64,
    ISize,
    U32,
    U64,
    USize,
    Bool,
    Char,
    Unit,
    Never,

    // String types
    Str,
    String,

    // Reference types
    Ref {
        lifetime: Option<String>,
        inner: Box<RustType>,
    },
    MutRef {
        lifetime: Option<String>,
        inner: Box<RustType>,
    },

    // Smart pointers
    Box(Box<RustType>),
    Rc(Box<RustType>),
    Arc(Box<RustType>),

    // Generic types
    Vec(Box<RustType>),
    Option(Box<RustType>),
    Result {
        ok: Box<RustType>,
        err: Box<RustType>,
    },

    // Slice type
    Slice {
        inner: Box<RustType>,
    },

    // Custom types
    Custom {
        name: String,
        generics: Vec<RustType>,
    },

    // Generic parameter
    Generic(String),
}

/// How a method takes self
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelfKind {
    None,                   // Associated function (no self)
    Value,                  // self
    Ref(Option<String>),    // &self or &'a self
    MutRef(Option<String>), // &mut self or &'a mut self
}

/// Information about an imported method with full signature details
#[derive(Debug, Clone)]
pub struct ImportedMethodInfo {
    pub _method_name: String,
    pub self_kind: SelfKind,
    pub _parameters: Vec<RustType>,
    pub return_type: RustType, // The actual parsed return type from Rust
    pub _trait_name: Option<String>, // Which trait this method comes from (if any)
}

impl RustType {
    /// Convert RustType to its Rust syntax representation
    pub fn to_rust_syntax(&self) -> String {
        match self {
            RustType::I32 => "i32".to_string(),
            RustType::I64 => "i64".to_string(),
            RustType::ISize => "isize".to_string(),
            RustType::U32 => "u32".to_string(),
            RustType::U64 => "u64".to_string(),
            RustType::USize => "usize".to_string(),
            RustType::Bool => "bool".to_string(),
            RustType::Char => "char".to_string(),
            RustType::Unit => "()".to_string(),
            RustType::Never => "!".to_string(),
            RustType::Str => "str".to_string(),
            RustType::String => "String".to_string(),
            RustType::Ref { lifetime, inner } => {
                if let Some(lt) = lifetime {
                    format!("&{} {}", lt, inner.to_rust_syntax())
                } else {
                    format!("&{}", inner.to_rust_syntax())
                }
            }
            RustType::MutRef { lifetime, inner } => {
                if let Some(lt) = lifetime {
                    format!("&{} mut {}", lt, inner.to_rust_syntax())
                } else {
                    format!("&mut {}", inner.to_rust_syntax())
                }
            }
            RustType::Box(inner) => format!("Box<{}>", inner.to_rust_syntax()),
            RustType::Rc(inner) => format!("Rc<{}>", inner.to_rust_syntax()),
            RustType::Arc(inner) => format!("Arc<{}>", inner.to_rust_syntax()),
            RustType::Vec(inner) => format!("Vec<{}>", inner.to_rust_syntax()),
            RustType::Option(inner) => format!("Option<{}>", inner.to_rust_syntax()),
            RustType::Result { ok, err } => {
                format!("Result<{}, {}>", ok.to_rust_syntax(), err.to_rust_syntax())
            }
            RustType::Custom { name, generics } => {
                if generics.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{}<{}>",
                        name,
                        generics
                            .iter()
                            .map(|g| g.to_rust_syntax())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            RustType::Generic(name) => name.clone(),
            RustType::Slice { inner } => {
                format!("[{}]", inner.to_rust_syntax())
            }
        }
    }

    /// Convert a Rust type to a Veltrano type
    pub fn to_veltrano_type(&self) -> Result<VeltranoType, String> {
        match self {
            // Primitives
            RustType::I32 => Ok(VeltranoType::i32()),
            RustType::I64 => Ok(VeltranoType::i64()),
            RustType::ISize => Ok(VeltranoType::isize()),
            RustType::U32 => Ok(VeltranoType::u32()),
            RustType::U64 => Ok(VeltranoType::u64()),
            RustType::USize => Ok(VeltranoType::usize()),
            RustType::Char => Ok(VeltranoType::char()),
            RustType::Bool => Ok(VeltranoType::bool()),
            RustType::Unit => Ok(VeltranoType::unit()),
            RustType::Never => Ok(VeltranoType::nothing()),

            // String types
            RustType::Str => Ok(VeltranoType::own(VeltranoType::str())), // Rust's str as return type is owned
            RustType::String => Ok(VeltranoType::own(VeltranoType::string())), // Rust's String is owned

            // References
            RustType::Ref { inner, .. } => {
                let inner_type = inner.to_veltrano_type()?;
                // Ref cancels out with Own
                if let TypeConstructor::Own = inner_type.constructor {
                    if let Some(inner_inner) = inner_type.args.first() {
                        Ok(inner_inner.clone())
                    } else {
                        Ok(VeltranoType::ref_(inner_type))
                    }
                } else {
                    Ok(VeltranoType::ref_(inner_type))
                }
            }
            RustType::MutRef { inner, .. } => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::mut_ref(inner_type))
            }

            // Smart pointers (owned)
            RustType::Box(inner) => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::own(VeltranoType::boxed(inner_type)))
            }

            // Generic types (owned containers)
            RustType::Vec(inner) => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::own(VeltranoType::vec(inner_type)))
            }
            RustType::Option(inner) => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::own(VeltranoType::option(inner_type)))
            }
            RustType::Result { ok, err } => {
                let ok_type = ok.to_veltrano_type()?;
                let err_type = err.to_veltrano_type()?;
                Ok(VeltranoType::own(VeltranoType::result(ok_type, err_type)))
            }

            // Custom types
            RustType::Custom { name, generics } => {
                // Special handling for known generic types without explicit generics
                match name.as_str() {
                    "Vec" if generics.is_empty() => {
                        // Vec without generics -> Vec<$T> (generic T)
                        Ok(VeltranoType::own(VeltranoType::vec(VeltranoType::custom(
                            "$T".to_string(),
                        ))))
                    }
                    _ => Ok(VeltranoType::custom(name.clone())),
                }
            }

            // Generic parameters
            RustType::Generic(name) => Ok(VeltranoType::custom(format!("${}", name))), // Prefix with $ to indicate generic

            // Slice type
            RustType::Slice { inner } => {
                let inner_type = inner.to_veltrano_type()?;
                Ok(VeltranoType::slice(inner_type))
            }

            _ => Err(format!("Unsupported Rust type for conversion: {:?}", self)),
        }
    }
}
