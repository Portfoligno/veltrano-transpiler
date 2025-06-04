# Veltrano Type System Design

## Overview

This document outlines the design for a **supplementary type system** for Veltrano that adds stricter type checking on top of Rust's existing type system. The goal is to create **layered type validation** where Veltrano enforces additional constraints before code reaches Rust's type checker, resulting in a more restrictive and safer type system overall.

## Philosophy: Layered Type Checking

```
┌─────────────────────────────┐
│    Veltrano Type System     │  <- Stricter, language-specific rules
│    (Supplementary Layer)    │
├─────────────────────────────┤
│      Rust Type System       │  <- Foundation layer (current status)
│    (Underlying Foundation)  │
└─────────────────────────────┘
```

**Key Principle**: Veltrano type checking happens **before** code generation. If Veltrano's type checker passes, the generated Rust code is guaranteed to type-check successfully, but Veltrano enforces additional restrictions that Rust alone would not.

## Veltrano-Specific Type Rules

### 1. Reference Depth Validation
```veltrano
// Veltrano enforces explicit reference management
val str1: Str = "hello"     // ✓ Veltrano: OK, Rust: &str
val str2: Own<Str> = str1   // ✗ Veltrano: Error - depth mismatch
val str3: Own<Str> = str1.own()  // ✓ Veltrano: OK after explicit conversion
```

### 2. Lifetime Scope Validation
```veltrano
fun<@caller> createPerson(name: Str@caller): Person@caller {
    val person: Own<Person> = Person(name = name, age = 30)
    return person.ref()  // ✓ Veltrano: lifetime @caller flows correctly
}

fun invalidLifetime(name: Str): Person@invalidLifetime {
    val person: Own<Person> = Person(name = name, age = 30)
    return person.ref()  // ✗ Veltrano: Error - @invalidLifetime cannot escape function
}

fun<@caller> returnOwned(name: Str@caller): Own<Person> {
    val person = Person(name = name, age = 30)
    return person  // ✓ Veltrano: returning owned value is always safe
}
```

### 3. Bump Allocation Constraints
```veltrano
fun<@caller> processData(data: Str@caller): Ref<ProcessedData@caller> {
    val processed: Own<ProcessedData> = ProcessedData(content = data)
    return processed.bumpRef()  // ✓ Veltrano: bump allocation in @caller lifetime
}

fun invalidBump(data: Str): Ref<ProcessedData@invalidBump> {
    val processed: Own<ProcessedData> = ProcessedData(content = data)
    return processed.bumpRef()  // ✗ Veltrano: Error - @invalidBump cannot escape
}
```

### 4. Method Availability Validation
```veltrano
val num: Int = 42
val numRef = num.ref()      // ✓ Veltrano: Int can be referenced
val numBump = num.bumpRef() // ✗ Veltrano: Error - Int doesn't need bump allocation

val str: Str = "hello"
val strRef = str.ref()      // ✓ Veltrano: Str can be referenced  
val strBump = str.bumpRef() // ✓ Veltrano: Str supports bump allocation
```

### 5. Data Class Initialization Validation
```veltrano
data class Person(val name: Str, val age: Int)

val person1: Own<Person> = Person(name = "Alice", age = 30)    // ✓ All fields provided
val person2: Own<Person> = Person(name = "Bob")                // ✗ Veltrano: Missing required field 'age'
val person3: Own<Person> = Person(name = "Charlie", extra = 1) // ✗ Veltrano: Unknown field 'extra'

// Type inference understands constructor returns Own<T>
val person4 = Person(name = "David", age = 40)  // Inferred as Own<Person>
val personRef: Person = person4.ref()           // Convert to reference type
```

## Core Components

### 1. Veltrano Type Checker
```rust
pub struct VeltranoTypeChecker {
    pub environment: TypeEnvironment,
    pub rules: Vec<TypeRule>,
    pub errors: Vec<TypeCheckError>,
}

impl VeltranoTypeChecker {
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeCheckError>>;
    pub fn check_statement(&mut self, stmt: &Stmt) -> Result<(), TypeCheckError>;
    pub fn check_expression(&mut self, expr: &Expr) -> Result<VeltranoType, TypeCheckError>;
}
```

### 2. Veltrano Type System
```rust
#[derive(Debug, Clone)]
pub struct VeltranoType {
    pub base: BaseType,
    pub reference_depth: u32,
    pub lifetime: Option<String>,      // Lifetime label (e.g., "@caller", "@a")
    pub mutability: Mutability,
    pub source_location: SourceLocation,
}

// For nested reference types like Ref@a<Str@b>
#[derive(Debug, Clone)]
pub struct NestedReferenceType {
    pub outer_lifetime: Option<String>,  // @a in Ref@a<...>
    pub inner_type: VeltranoType,       // Str@b
}

// Data class constructors always return Own<T>
impl VeltranoType {
    pub fn data_class_constructor(class_name: &str) -> Self {
        VeltranoType {
            base: BaseType::Custom(class_name.to_string()),
            reference_depth: 0,  // Own<T> has depth 0
            lifetime: None,
            mutability: Mutability::Owned,
            source_location: SourceLocation::default(),
        }
    }
    
    pub fn with_lifetime(mut self, lifetime: String) -> Self {
        self.lifetime = Some(lifetime);
        self
    }
}

#[derive(Debug, Clone)]
pub enum TypeRule {
    ReferenceDepthConsistency,    // Own<T> + Ref<T> operations must match depths
    LifetimeEscapeValidation,     // Prevent @local lifetimes from escaping
    BumpAllocationConstraints,    // Only reference types can use .bumpRef()
    MethodAvailabilityCheck,      // .ref(), .clone(), etc. only on valid types
    DataClassFieldValidation,     // All required fields must be provided
    ExplicitConversionRequired,   // No implicit reference depth changes
}
```

### 3. Type Error Reporting
```rust
#[derive(Debug, Clone)]
pub enum TypeCheckError {
    ReferenceDepthMismatch { 
        expected: u32, 
        found: u32, 
        location: SourceLocation 
    },
    LifetimeEscapeError { 
        lifetime: String, 
        escape_location: SourceLocation 
    },
    InvalidMethodCall { 
        method: String, 
        type_name: String, 
        location: SourceLocation 
    },
    MissingRequiredField { 
        field: String, 
        class: String, 
        location: SourceLocation 
    },
    UnknownField { 
        field: String, 
        class: String, 
        location: SourceLocation 
    },
}
```

## Implementation Strategy

### Phase 1: Core Infrastructure
1. **Extend AST** with source location tracking for better error messages
2. **Create VeltranoType** representation with Veltrano-specific metadata
3. **Build TypeEnvironment** for tracking variable and function types
4. **Implement basic type rules** as modular validators

### Phase 2: Reference Depth Validation  
1. **Track reference depths** through expressions and assignments
2. **Validate .ref() and .own() operations** for depth consistency
3. **Enforce explicit conversions** between different reference depths
4. **Add depth-aware method availability** checking

### Phase 3: Lifetime Validation
1. **Parse lifetime syntax** - `fun<@a, @b>` declarations and `Type@lifetime` annotations
2. **Track lifetime scopes** including implicit function lifetimes (e.g., `@functionName`)
3. **Integrate with bump allocation** - each lifetime has associated bump allocator
4. **Validate lifetime flow** and prevent function-local lifetimes from escaping

### Phase 4: Advanced Validation
1. **Data class field validation** for complete initialization
2. **Method availability checking** based on type properties
3. **Integration with existing codegen** to preserve all validation
4. **Comprehensive error messages** with suggestions

## Example Validation Flow

### Input Veltrano Code
```veltrano
fun<@caller> processName(input: Str@caller): Ref<Str@caller> {
    val processed = input.toUpperCase()
    return processed.bumpRef()
}
```

### Veltrano Type Checking Steps
1. **Parse lifetime parameter** `<@caller>` in function declaration
2. **Validate toUpperCase() method** exists for Str type  
3. **Check bumpRef() availability** for String type (✓ allowed)
4. **Validate lifetime flow** @caller → processed → return (✓ valid)
5. **Verify return type** matches function signature (✓ Ref<Str@caller>)

### Generated Rust Code (if validation passes)
```rust
fn process_name<'a>(bump: &'a bumpalo::Bump, input: &'a str) -> &'a str {
    let processed = input.to_uppercase();
    bump.alloc(processed)
}
```

### What Rust Alone Would Allow (but Veltrano Prevents)
```veltrano
fun invalidExample(input: Str): Ref<Str@invalidExample> {
    return input.bumpRef()  // ✗ Veltrano: @invalidExample cannot escape function
}
// Rust would generate code that compiles but violates Veltrano's safety model
```

## Benefits of Layered Approach

### 1. Stricter Safety
- **Earlier error detection** before Rust compilation
- **Language-specific constraints** that Rust cannot enforce
- **Clearer error messages** in Veltrano terms, not Rust terms

### 2. Language Control
- **Enforce Veltrano idioms** like explicit reference management
- **Prevent unsafe patterns** that Rust allows but Veltrano shouldn't
- **Guide developers** toward correct Veltrano usage patterns

### 3. Evolution Path
- **Independent development** of Veltrano type rules
- **Backward compatibility** with existing Rust type system
- **Foundation for future features** like Veltrano-specific generics

### 4. Developer Experience
- **Faster feedback** than waiting for Rust compiler
- **Context-aware suggestions** for fixing Veltrano-specific issues
- **Educational value** by explaining Veltrano's type model

## Implementation Notes

### Type Checking Phases
1. **Pre-parsing**: Syntax-level validation
2. **AST building**: Structure validation with source locations
3. **Veltrano type checking**: Language-specific rule validation
4. **Code generation**: Produce Rust code (guaranteed to type-check)
5. **Rust compilation**: Final validation layer

### Error Handling Strategy
- **Fail fast**: Stop at first Veltrano type error
- **Collect multiple errors**: Show all issues in a statement/expression
- **Suggest fixes**: Provide concrete suggestions for common mistakes
- **Source locations**: Precise error location reporting

### Integration Points
- **Parser integration**: Add source location tracking
- **AST enhancement**: Include type metadata in nodes
- **Codegen integration**: Use validated type information
- **CLI integration**: Report type errors before Rust compilation

## Next Steps

1. **Implement core type infrastructure** with source locations
2. **Build reference depth validation** rules
3. **Add lifetime scope tracking** and validation
4. **Create comprehensive error reporting** system
5. **Integrate with existing transpiler** pipeline
6. **Add extensive test coverage** for type rules
