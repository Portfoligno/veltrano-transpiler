# Veltrano Type System Design

## Overview

This document outlines the design for a comprehensive type system for the Veltrano transpiler. The type system will provide:

1. **Rust Type Signature Extraction** - Parse and extract type information from generated Rust code
2. **Type Inference** - Automatically infer types for Veltrano expressions
3. **Type Checking** - Validate type compatibility and catch errors
4. **Lifetime Integration** - Work seamlessly with existing lifetime and reference depth systems

## Core Components

### 1. Type Representation

#### Enhanced Type Structure
```rust
#[derive(Debug, Clone)]
pub struct Type {
    pub base: BaseType,
    pub reference_depth: u32,
    pub lifetime: Option<String>,     // Optional lifetime parameter (e.g., "'a")
    pub inferred: bool,               // Whether this type was inferred
}

#[derive(Debug, Clone)]
pub struct TypeSignature {
    pub rust_type: String,            // Actual Rust type (e.g., "&'a str")
    pub veltrano_type: Type,          // Corresponding Veltrano type
    pub constraints: Vec<TypeConstraint>, // Type constraints
}
```

#### Type Constraints
```rust
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    Equal(Type, Type),                // Two types must be equal
    RefOf(Type, Type),                // First type is reference to second
    Clone(Type),                      // Type must support Clone
    HasMethod(Type, String),          // Type must have specific method
    BumpCompatible(Type),             // Type must work with bump allocation
}
```

### 2. Type Environment

#### Context Tracking
```rust
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    pub variables: HashMap<String, TypeSignature>,
    pub functions: HashMap<String, FunctionSignature>,
    pub data_classes: HashMap<String, DataClassSignature>,
    pub current_lifetime: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<(String, TypeSignature)>,
    pub return_type: TypeSignature,
    pub has_bump_parameter: bool,
    pub lifetime_parameters: Vec<String>,
}
```

### 3. Rust Type Parser

#### Type Signature Extraction
```rust
pub struct RustTypeParser {
    // Parser for extracting type information from Rust code
}

impl RustTypeParser {
    pub fn parse_type_signature(rust_code: &str) -> Result<TypeSignature, ParseError>;
    pub fn extract_function_signatures(rust_code: &str) -> Vec<FunctionSignature>;
    pub fn extract_struct_signatures(rust_code: &str) -> Vec<DataClassSignature>;
}
```

### 4. Type Inference Engine

#### Inference Algorithm
```rust
pub struct TypeInferenceEngine {
    pub environment: TypeEnvironment,
    pub constraints: Vec<TypeConstraint>,
}

impl TypeInferenceEngine {
    pub fn infer_expression_type(&mut self, expr: &Expr) -> Result<TypeSignature, InferenceError>;
    pub fn infer_statement_types(&mut self, stmt: &Stmt) -> Result<(), InferenceError>;
    pub fn solve_constraints(&mut self) -> Result<(), ConstraintError>;
}
```

## Implementation Strategy

### Phase 1: Core Type Infrastructure
1. **Extend Type struct** with lifetime and inference metadata
2. **Add TypeSignature and TypeConstraint** enums
3. **Create TypeEnvironment** for context tracking
4. **Implement basic type methods** (to_rust_signature, needs_lifetime, etc.)

### Phase 2: Rust Type Parsing
1. **Simple type parser** for basic Rust types (i64, bool, &str, etc.)
2. **Struct signature extraction** from generated data classes
3. **Function signature parsing** from generated functions
4. **Lifetime parameter extraction** from complex types

### Phase 3: Type Inference
1. **Literal type inference** (integers, strings, booleans)
2. **Variable type tracking** in TypeEnvironment
3. **Method call type inference** (.ref(), .bumpRef(), .clone(), etc.)
4. **Function call type inference** with parameter matching

### Phase 4: Integration
1. **Integrate with existing codegen** to use type information
2. **Enhance error messages** with type information
3. **Type-aware optimizations** in code generation
4. **Lifetime inference improvements** using type constraints

## Key Design Decisions

### 1. Rust Type Signature as Source of Truth
- Generated Rust code contains the authoritative type information
- Type inference works backward from desired Rust output
- Enables precise type checking and better error messages

### 2. Constraint-Based Inference
- Type constraints capture relationships between types
- Constraint solver resolves complex type relationships
- Supports advanced features like method overloading resolution

### 3. Lifetime Integration
- Seamless integration with existing lifetime system
- Type system understands bump allocation patterns
- Automatic lifetime parameter inference for data classes

### 4. Incremental Implementation
- Start with simple types and build complexity gradually
- Maintain compatibility with existing transpiler functionality
- Add type checking as optional validation layer initially

## Example Use Cases

### 1. Basic Type Inference
```veltrano
val x = 42        // Inferred as Int (i64 in Rust)
val s = "hello"   // Inferred as Str (&str in Rust)
val b = true      // Inferred as Bool (bool in Rust)
```

### 2. Method Chain Type Tracking
```veltrano
val person = Person(name = "Alice", age = 30)  // Person<'a>
val nameRef = person.name.ref()                // &str (from &'a str)
val cloned = person.clone()                    // Person<'a>
```

### 3. Function Type Checking
```veltrano
fun greet(name: Str): Str {
    return "Hello, " + name    // Type check: String concatenation
}

val greeting = greet("World")  // Type check: argument compatibility
```

### 4. Data Class Type Inference
```veltrano
data class Book(val title: Str, val pages: Int)

val book = Book(title = "Rust Guide", pages = 300)
// Inferred: Book<'a> with lifetime for title field
```

## Benefits

### 1. Enhanced Developer Experience
- **Better error messages** with precise type information
- **IDE integration** potential with type information
- **Safer code** through compile-time type checking

### 2. Improved Code Generation
- **Type-aware optimizations** in Rust output
- **Automatic lifetime inference** for complex patterns
- **Better reference handling** with type constraints

### 3. Language Evolution
- **Foundation for generics** in future Veltrano versions
- **Support for traits** and interfaces
- **Advanced type features** like associated types

### 4. Debugging and Analysis
- **Type information in AST** for tools and analysis
- **Constraint visualization** for debugging inference
- **Performance analysis** with type-aware metrics

## Next Steps

1. **Create basic type infrastructure** (Phase 1)
2. **Implement simple Rust type parser** (Phase 2)
3. **Build constraint-based inference engine** (Phase 3)
4. **Integrate with existing transpiler** (Phase 4)
5. **Add comprehensive test cases** and examples
6. **Document type system usage** for language users
