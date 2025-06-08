/// Core type system definitions for Veltrano
///
/// This module contains the fundamental type definitions that are shared
/// across multiple modules to avoid circular dependencies.
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
    pub fn to_rust_type_name(&self) -> String {
        match &self.constructor {
            TypeConstructor::I32 => "i32".to_string(),
            TypeConstructor::I64 => "i64".to_string(),
            TypeConstructor::ISize => "isize".to_string(),
            TypeConstructor::U32 => "u32".to_string(),
            TypeConstructor::U64 => "u64".to_string(),
            TypeConstructor::USize => "usize".to_string(),
            TypeConstructor::Bool => "bool".to_string(),
            TypeConstructor::Char => "char".to_string(),
            TypeConstructor::Str => "&str".to_string(),
            TypeConstructor::String => "String".to_string(),
            TypeConstructor::Unit => "()".to_string(),
            TypeConstructor::Nothing => "!".to_string(),
            TypeConstructor::Custom(name) => name.clone(),
            // For container types, delegate to their inner type
            TypeConstructor::Own => {
                if let Some(inner) = self.inner() {
                    inner.to_rust_type_name()
                } else {
                    "unknown".to_string()
                }
            }
            TypeConstructor::Ref => {
                if let Some(inner) = self.inner() {
                    format!("&{}", inner.to_rust_type_name())
                } else {
                    "unknown".to_string()
                }
            }
            TypeConstructor::MutRef => {
                if let Some(inner) = self.inner() {
                    format!("&mut {}", inner.to_rust_type_name())
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }

    /// Get the ultimate base type constructor (recursively unwrap constructors)
    pub fn get_base_constructor(&self) -> &TypeConstructor {
        if self.args.is_empty() {
            &self.constructor
        } else if let Some(inner) = self.inner() {
            inner.get_base_constructor()
        } else {
            &self.constructor
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
pub struct MethodSignature {
    pub name: String,
    pub receiver_type: VeltranoType,
    pub parameters: Vec<VeltranoType>,
    pub return_type: VeltranoType,
}

/// Source location for error reporting
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
}

/// Data class definition with field information
#[derive(Debug, Clone)]
pub struct DataClassDefinition {
    pub name: String,
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
