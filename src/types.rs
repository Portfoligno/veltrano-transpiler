/// Core type system definitions for Veltrano
///
/// This module contains the fundamental type definitions that are shared
/// across multiple modules to avoid circular dependencies.
use crate::rust_interop::RustType;
use std::collections::HashMap;

/// A type in the Veltrano type system supporting higher-kinded types
#[derive(Debug, Clone, PartialEq)]
pub struct VeltranoType {
    /// The type constructor or base type
    pub constructor: TypeConstructor,
    /// Type arguments (empty for base types)
    pub args: Vec<VeltranoType>,
}

/// Type constructors and base types with their kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstructor {
    // Base types (kind *)
    // Signed integers
    /// i32 in Rust
    I32,
    /// i64 in Rust
    I64,
    /// isize in Rust
    ISize,
    // Unsigned integers
    /// u32 in Rust
    U32,
    /// u64 in Rust
    U64,
    /// usize in Rust
    USize,
    // Other primitives
    /// bool in Rust
    Bool,
    /// char in Rust
    Char,
    /// () in Rust
    Unit,
    /// ! in Rust (never type)
    Nothing,
    /// &str in Rust (string slice)
    Str,
    /// &String in Rust (reference to owned string)
    String,
    /// Custom/user-defined types
    Custom(String),

    /// Generic type parameter with constraints
    /// e.g., Generic("T", vec!["Clone"]) represents T: Clone
    Generic(String, Vec<String>),

    // Built-in type constructors (kind * -> *)
    /// Own<T> - forces ownership, removes reference level for reference types
    Own,
    /// Ref<T> - adds reference level (&T)
    Ref,
    /// MutRef<T> - mutable reference (&mut T)
    MutRef,
    /// Box<T> - heap allocation
    Box,
    /// Vec<T> - dynamic array
    Vec,
    /// Option<T> - optional value
    Option,

    // Higher-kinded constructors (kind * -> * -> *)
    /// Result<T, E> - result type
    Result,

    // Special cases
    /// Array<T, N> - fixed-size array (size is part of type)
    Array(usize),
    /// Slice<T> - dynamically sized slice type (&[T] in Rust)
    Slice,
}

impl VeltranoType {
    /// Helper constructors for base types
    // Signed integers
    pub fn i32() -> Self {
        Self {
            constructor: TypeConstructor::I32,
            args: vec![],
        }
    }

    pub fn i64() -> Self {
        Self {
            constructor: TypeConstructor::I64,
            args: vec![],
        }
    }

    pub fn isize() -> Self {
        Self {
            constructor: TypeConstructor::ISize,
            args: vec![],
        }
    }

    // Unsigned integers
    pub fn u32() -> Self {
        Self {
            constructor: TypeConstructor::U32,
            args: vec![],
        }
    }

    pub fn u64() -> Self {
        Self {
            constructor: TypeConstructor::U64,
            args: vec![],
        }
    }

    pub fn usize() -> Self {
        Self {
            constructor: TypeConstructor::USize,
            args: vec![],
        }
    }

    // Other primitives
    pub fn bool() -> Self {
        Self {
            constructor: TypeConstructor::Bool,
            args: vec![],
        }
    }

    pub fn char() -> Self {
        Self {
            constructor: TypeConstructor::Char,
            args: vec![],
        }
    }

    pub fn unit() -> Self {
        Self {
            constructor: TypeConstructor::Unit,
            args: vec![],
        }
    }

    pub fn nothing() -> Self {
        Self {
            constructor: TypeConstructor::Nothing,
            args: vec![],
        }
    }

    pub fn str() -> Self {
        Self {
            constructor: TypeConstructor::Str,
            args: vec![],
        }
    }

    pub fn string() -> Self {
        Self {
            constructor: TypeConstructor::String,
            args: vec![],
        }
    }

    pub fn custom(name: String) -> Self {
        Self {
            constructor: TypeConstructor::Custom(name),
            args: vec![],
        }
    }

    pub fn generic(name: String, constraints: Vec<String>) -> Self {
        Self {
            constructor: TypeConstructor::Generic(name, constraints),
            args: vec![],
        }
    }

    /// Apply type constructors
    pub fn own(inner: VeltranoType) -> Self {
        // Note: Validation is now handled during type checking phase
        // This constructor only creates the type representation
        Self {
            constructor: TypeConstructor::Own,
            args: vec![inner],
        }
    }

    pub fn ref_(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Ref,
            args: vec![inner],
        }
    }

    pub fn mut_ref(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::MutRef,
            args: vec![inner],
        }
    }

    pub fn vec(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Vec,
            args: vec![inner],
        }
    }

    pub fn boxed(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Box,
            args: vec![inner],
        }
    }

    pub fn array(inner: VeltranoType, size: usize) -> Self {
        Self {
            constructor: TypeConstructor::Array(size),
            args: vec![inner],
        }
    }

    pub fn slice(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Slice,
            args: vec![inner],
        }
    }

    pub fn option(inner: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Option,
            args: vec![inner],
        }
    }

    pub fn result(ok_type: VeltranoType, err_type: VeltranoType) -> Self {
        Self {
            constructor: TypeConstructor::Result,
            args: vec![ok_type, err_type],
        }
    }

    /// Compatibility methods for migration
    pub fn inner(&self) -> Option<&VeltranoType> {
        self.args.first()
    }

    /// Convert this VeltranoType to its corresponding Rust type name
    /// This method requires a trait checker to determine if types implement Copy

    /// Convert VeltranoType to RustType
    pub fn to_rust_type(
        &self,
        trait_checker: &mut crate::rust_interop::RustInteropRegistry,
    ) -> crate::rust_interop::RustType {
        self.to_rust_type_with_lifetime(trait_checker, None)
    }

    /// Convert VeltranoType to RustType with optional lifetime
    pub fn to_rust_type_with_lifetime(
        &self,
        trait_checker: &mut crate::rust_interop::RustInteropRegistry,
        lifetime: Option<String>,
    ) -> crate::rust_interop::RustType {
        use crate::rust_interop::RustType;

        match &self.constructor {
            TypeConstructor::I32 => RustType::I32,
            TypeConstructor::I64 => RustType::I64,
            TypeConstructor::ISize => RustType::ISize,
            TypeConstructor::U32 => RustType::U32,
            TypeConstructor::U64 => RustType::U64,
            TypeConstructor::USize => RustType::USize,
            TypeConstructor::Bool => RustType::Bool,
            TypeConstructor::Char => RustType::Char,
            TypeConstructor::Unit => RustType::Unit,
            TypeConstructor::Nothing => RustType::Never,
            TypeConstructor::Str => RustType::Ref {
                lifetime: lifetime.clone(),
                inner: Box::new(RustType::Str),
            },
            TypeConstructor::String => {
                // String is naturally referenced in Veltrano
                RustType::Ref {
                    lifetime: lifetime.clone(),
                    inner: Box::new(RustType::String),
                }
            }
            TypeConstructor::Custom(name) => {
                if self.implements_copy(trait_checker) {
                    RustType::Custom {
                        name: name.clone(),
                        generics: vec![],
                    }
                } else {
                    // Naturally referenced custom types
                    RustType::Ref {
                        lifetime: lifetime.clone(),
                        inner: Box::new(RustType::Custom {
                            name: name.clone(),
                            generics: vec![],
                        }),
                    }
                }
            }
            TypeConstructor::Generic(name, _constraints) => {
                // Generic types are treated as custom types for Rust generation
                // The actual type will be substituted during type checking
                RustType::Custom {
                    name: name.clone(),
                    generics: vec![],
                }
            }
            TypeConstructor::Own => {
                if let Some(inner) = self.inner() {
                    // Own<T> removes one level of reference
                    let inner_rust = inner.to_rust_type_with_lifetime(trait_checker, lifetime);
                    match inner_rust {
                        RustType::Ref { inner, .. } => *inner,
                        other => other,
                    }
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Ref => {
                if let Some(inner) = self.inner() {
                    RustType::Ref {
                        lifetime: lifetime.clone(),
                        inner: Box::new(inner.to_rust_type_with_lifetime(trait_checker, lifetime)),
                    }
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::MutRef => {
                if let Some(inner) = self.inner() {
                    RustType::MutRef {
                        lifetime: lifetime.clone(),
                        inner: Box::new(inner.to_rust_type_with_lifetime(trait_checker, lifetime)),
                    }
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Box => {
                if let Some(inner) = self.inner() {
                    // Box is naturally referenced (like non-Copy custom types)
                    RustType::Ref {
                        lifetime: lifetime.clone(),
                        inner: Box::new(RustType::Box(Box::new(
                            inner.to_rust_type_with_lifetime(trait_checker, lifetime.clone()),
                        ))),
                    }
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Vec => {
                if let Some(inner) = self.inner() {
                    RustType::Vec(Box::new(
                        inner.to_rust_type_with_lifetime(trait_checker, lifetime),
                    ))
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Option => {
                if let Some(inner) = self.inner() {
                    RustType::Option(Box::new(
                        inner.to_rust_type_with_lifetime(trait_checker, lifetime),
                    ))
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Result => {
                if self.args.len() == 2 {
                    RustType::Result {
                        ok: Box::new(
                            self.args[0]
                                .to_rust_type_with_lifetime(trait_checker, lifetime.clone()),
                        ),
                        err: Box::new(
                            self.args[1].to_rust_type_with_lifetime(trait_checker, lifetime),
                        ),
                    }
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Array(size) => {
                // Arrays in Rust don't have a direct RustType representation
                // We'll need to handle this specially in codegen
                // For now, return a Custom type that represents the array
                if let Some(inner) = self.inner() {
                    RustType::Custom {
                        name: format!("[_; {}]", size), // Placeholder
                        generics: vec![inner.to_rust_type_with_lifetime(trait_checker, lifetime)],
                    }
                } else {
                    RustType::Never // Error case
                }
            }
            TypeConstructor::Slice => {
                // Slice<T> maps to &[T] in Rust
                if let Some(inner) = self.inner() {
                    RustType::Ref {
                        lifetime: lifetime.clone(),
                        inner: Box::new(RustType::Slice {
                            inner: Box::new(
                                inner.to_rust_type_with_lifetime(trait_checker, lifetime),
                            ),
                        }),
                    }
                } else {
                    RustType::Never // Error case
                }
            }
        }
    }

    /// Check if this type implements the Copy trait
    pub fn implements_copy(
        &self,
        trait_checker: &mut crate::rust_interop::RustInteropRegistry,
    ) -> bool {
        match &self.constructor {
            // Primitive types all implement Copy
            TypeConstructor::I32
            | TypeConstructor::I64
            | TypeConstructor::ISize
            | TypeConstructor::U32
            | TypeConstructor::U64
            | TypeConstructor::USize
            | TypeConstructor::Bool
            | TypeConstructor::Char
            | TypeConstructor::Unit => true,

            // Never type is Copy
            TypeConstructor::Nothing => true,

            // String types do not implement Copy
            TypeConstructor::Str | TypeConstructor::String => false,

            // References are Copy if their inner type is
            TypeConstructor::Ref | TypeConstructor::MutRef => {
                if let Some(inner) = self.inner() {
                    inner.implements_copy(trait_checker)
                } else {
                    false
                }
            }

            // Own, Box, Vec, etc. never implement Copy
            TypeConstructor::Own | TypeConstructor::Box | TypeConstructor::Vec => false,

            // Option and Result depend on their inner types
            TypeConstructor::Option => {
                if let Some(inner) = self.inner() {
                    inner.implements_copy(trait_checker)
                } else {
                    false
                }
            }
            TypeConstructor::Result => {
                // Result is Copy only if both Ok and Err types are Copy
                if self.args.len() == 2 {
                    self.args[0].implements_copy(trait_checker)
                        && self.args[1].implements_copy(trait_checker)
                } else {
                    false
                }
            }

            // Arrays are Copy if their element type is Copy
            TypeConstructor::Array(_) => {
                if let Some(inner) = self.inner() {
                    inner.implements_copy(trait_checker)
                } else {
                    false
                }
            }

            // Custom types need to check with the trait checker
            TypeConstructor::Custom(name) => {
                let rust_type = RustType::Custom {
                    name: name.clone(),
                    generics: vec![],
                };
                trait_checker
                    .type_implements_trait(&rust_type, "Copy")
                    .unwrap_or(false)
            }
            // Generic types - we can't know if they implement Copy without constraints
            // For now, assume they don't unless explicitly constrained
            TypeConstructor::Generic(_, constraints) => constraints.contains(&"Copy".to_string()),

            // Slices don't implement Copy (they're DSTs)
            TypeConstructor::Slice => false,
        }
    }
}


/// Function signature for type checking
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<VeltranoType>,
    pub return_type: VeltranoType,
}

/// Method signature for type checking
#[derive(Debug, Clone)]
pub struct _MethodSignature {
    pub name: String,
    pub receiver_type: VeltranoType,
    pub parameters: Vec<VeltranoType>,
    pub return_type: VeltranoType,
}

/// Data class definition with field information
#[derive(Debug, Clone)]
pub struct DataClassDefinition {
    pub _name: String,
    pub fields: Vec<DataClassFieldSignature>,
}

#[derive(Debug, Clone)]
pub struct DataClassFieldSignature {
    pub name: String,
    pub field_type: VeltranoType,
}

/// Type environment for tracking variables, functions, and data classes
pub struct TypeEnvironment {
    variables: HashMap<String, VeltranoType>,
    functions: HashMap<String, FunctionSignature>,
    data_classes: HashMap<String, DataClassDefinition>,
    scopes: Vec<HashMap<String, VeltranoType>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            data_classes: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn lookup_variable(&self, name: &str) -> Option<&VeltranoType> {
        // Check current scopes first (most recent first)
        for scope in self.scopes.iter().rev() {
            if let Some(var_type) = scope.get(name) {
                return Some(var_type);
            }
        }

        // Check global variables
        self.variables.get(name)
    }

    pub fn declare_variable(&mut self, name: String, typ: VeltranoType) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, typ);
        } else {
            self.variables.insert(name, typ);
        }
    }

    pub fn declare_function(&mut self, name: String, signature: FunctionSignature) {
        self.functions.insert(name, signature);
    }

    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare_data_class(&mut self, name: String, definition: DataClassDefinition) {
        self.data_classes.insert(name, definition);
    }

    pub fn lookup_data_class(&self, name: &str) -> Option<&DataClassDefinition> {
        self.data_classes.get(name)
    }
}
