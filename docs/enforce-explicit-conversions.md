# Enforcing Explicit Type Conversions in Veltrano

## Overview

This document outlines the implementation of **enforcing explicit type conversions** in Veltrano, which is the first step toward stricter type checking and explicit memory management.

This implementation represents **Phase 1** of the broader Veltrano type system described in [`type-system-design.md`](type-system-design.md). Specifically, it implements the **ExplicitConversionRequired** and **ReferenceDepthConsistency** type rules that prevent implicit conversions between different ownership levels.

## Table of Contents

1. [Core Concept](#core-concept) - What and why
2. [Implementation Approach](#implementation-approach) - High-level architecture
3. [Conversion Types Covered](#conversion-types-covered) - Complete overview with examples
4. [Implementation Details](#implementation-details) - Technical specifications
5. [Testing Strategy](#testing-strategy) - Unit and integration tests
6. [Success Criteria](#success-criteria) - Definition of done
7. [Migration Guide](#migration-path-for-existing-code) - How to update existing code

## Core Concept

### What are Implicit Type Conversions?

Implicit type conversions are automatic transformations between related types that happen without explicit syntax. In Rust, this includes Deref coercion and other automatic conversions:

```rust
// Rust allows these automatic conversions:
let owned: String = "hello".to_string();
let borrowed: &str = &owned;  // Automatic String → &str conversion
let slice: &[i32] = &vec![1, 2, 3];  // Automatic Vec<T> → &[T] conversion
```

### Why Require Explicit Conversions in Veltrano?

#### 1. **Explicit Memory Management**
Veltrano aims to make memory ownership and borrowing explicit, helping developers understand exactly when and how data is being referenced.

#### 2. **Prevent Hidden Allocations**
Automatic conversions can hide performance implications. Explicit conversions make the cost visible.

#### 3. **Clearer Type System**
By requiring explicit conversions, the type system becomes more predictable and easier to reason about.

#### 4. **Foundation for Advanced Features**
Requiring explicit conversions is a prerequisite for implementing lifetime annotations, bump allocation, and other Veltrano-specific features.

## Implementation Approach

**Core Principle**: Separate type checking from error analysis for simplicity and performance.

### Simple Architecture
1. **Strict Type Checker**: Enforces exact type matching everywhere (`actual_type == expected_type`)
2. **Error Analyzer**: When types don't match, suggests explicit conversions using fast pattern matching
3. **Clean Separation**: Type checking logic independent of error suggestions

### Key Benefits
- **Simple Implementation**: Type checker just does structural equality checks
- **Fast Performance**: No complex context tracking during type checking
- **Maintainable**: Clear responsibilities and straightforward code
- **Extensible**: Error suggestions can evolve independently

## Conversion Types Covered

Veltrano prevents implicit conversions in **all** contexts where they can occur. Here's a comprehensive overview:

### Core Type Conversions
| From | To | Explicit Method Required |
|------|----|-----------------------|
| `Own<T>` | `T` | `.ref()` |
| `String` | `Str` | `.ref()` |
| `Own<String>` | `Str` | `.ref().ref()` |
| `MutRef<T>` | `T` | `.ref()` |
| `Vec<T>` | `Slice<T>` | `.toSlice()` |
| `[T; N]` | `Slice<T>` | `.toSlice()` |

### Contexts Where Conversions Are Prevented
| Context | Example | Fix Required |
|---------|---------|-------------|
| **Variable Assignment** | `val s: String = owned` | `owned.ref()` |
| **Function Arguments** | `func(owned)` | `func(owned.ref())` |
| **Method Receivers** | `owned.method()` | `owned.ref().method()` |
| **Return Values** | `return owned` | `return owned.ref()` |
| **Field Access** | `obj.field` | `obj.ref().field` |
| **Indexing** | `arr[0]` | `arr.ref()[0]` |
| **Operators** | `a + b` | `a.ref() + b.ref()` |
| **Pattern Matching** | `Some(x) { ... }` | explicit conversion in body |
| **Generic Constraints** | trait bounds | explicit conversion |
| **Closures** | capture conversions | explicit conversion |

### Representative Examples

#### Before (Implicit - What We Prevent)
```veltrano
fun processString(s: Str): Int { return s.length() }

fun main() {
    val owned: Own<String> = "hello".toString()
    
    // ❌ All these require implicit conversions
    val borrowed: String = owned              // Assignment
    val len = processString(owned)            // Function argument  
    val upper = owned.toUpperCase()           // Method call
    val char = owned[0]                       // Indexing
}
```

#### After (Explicit - What We Require)
```veltrano
fun main() {
    val owned: Own<String> = "hello".toString()
    
    // ✅ All conversions must be explicit
    val borrowed: String = owned.ref()              // Assignment
    val len = processString(owned.ref().ref())      // Function argument
    val upper = owned.ref().toUpperCase()           // Method call  
    val char = owned.ref()[0]                       // Indexing
}
```

## Implementation Details

The key insight is to **separate type checking from error analysis**. The type checker should be simple and strict, while error suggestions are generated separately using fast heuristics.

### Phase 1: Strict Type Checking

#### 1.1 Simple Type System
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct VeltranoType {
    pub base: BaseType,
    pub ownership: Ownership,
    pub mutability: Mutability,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Owned,      // Own<T> - equivalent to T in Rust
    Borrowed,   // T - equivalent to &T in Rust  
    MutBorrowed, // MutRef<T> - equivalent to &mut T in Rust
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Int,
    Bool,
    Str,
    String,
    Unit,
    Vec(Box<VeltranoType>),          // Vec<T> - owned dynamic arrays
    Slice(Box<VeltranoType>),        // Slice<T> - borrowed array views (&[T] in Rust)
    Array(Box<VeltranoType>, usize), // [T; N] - fixed-size arrays
    Custom(String),
    // ... other types
}
```

#### 1.2 Type Environment
```rust
pub struct TypeEnvironment {
    variables: HashMap<String, VeltranoType>,
    functions: HashMap<String, FunctionSignature>,
    scopes: Vec<HashMap<String, VeltranoType>>,
}

impl TypeEnvironment {
    pub fn lookup_variable(&self, name: &str) -> Option<&VeltranoType>;
    pub fn declare_variable(&mut self, name: String, typ: VeltranoType);
    pub fn enter_scope(&mut self);
    pub fn exit_scope(&mut self);
}
```

### Phase 2: Strict Type Checking (No Implicit Conversion Logic)

The core principle: **The type checker is agnostic about implicit conversions and simply enforces exact type matching everywhere.**

#### 2.1 Simple Type Checker
```rust
impl VeltranoTypeChecker {
    fn check_assignment(&mut self, expected: &VeltranoType, value: &Expr) -> Result<(), TypeCheckError> {
        let actual = self.check_expression(value)?;
        
        if !self.types_equal(expected, &actual) {
            return Err(TypeCheckError::TypeMismatch {
                expected: expected.clone(),
                actual,
                location: value.location(),
            });
        }
        
        Ok(())
    }
    
    fn check_function_call(&mut self, func_name: &str, args: &[Expr]) -> Result<VeltranoType, TypeCheckError> {
        let func_sig = self.lookup_function(func_name)?;
        
        if args.len() != func_sig.parameters.len() {
            return Err(TypeCheckError::ArgumentCountMismatch {
                function: func_name.to_string(),
                expected: func_sig.parameters.len(),
                actual: args.len(),
                location: /* ... */,
            });
        }
        
        for (i, (arg, expected_param)) in args.iter().zip(&func_sig.parameters).enumerate() {
            let actual_param = self.check_expression(arg)?;
            
            if !self.types_equal(expected_param, &actual_param) {
                return Err(TypeCheckError::TypeMismatch {
                    expected: expected_param.clone(),
                    actual: actual_param,
                    location: arg.location(),
                });
            }
        }
        
        Ok(func_sig.return_type.clone())
    }
    
    fn check_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr]) -> Result<VeltranoType, TypeCheckError> {
        let receiver_type = self.check_expression(receiver)?;
        
        // Look up method on the exact receiver type (no implicit conversions)
        let method_sig = self.lookup_method(&receiver_type, method)
            .ok_or_else(|| TypeCheckError::MethodNotFound {
                receiver_type: receiver_type.clone(),
                method: method.to_string(),
                location: receiver.location(),
            })?;
        
        // Check method arguments with strict type matching
        for (i, (arg, expected_param)) in args.iter().zip(&method_sig.parameters).enumerate() {
            let actual_param = self.check_expression(arg)?;
            
            if !self.types_equal(expected_param, &actual_param) {
                return Err(TypeCheckError::TypeMismatch {
                    expected: expected_param.clone(),
                    actual: actual_param,
                    location: arg.location(),
                });
            }
        }
        
        Ok(method_sig.return_type.clone())
    }
    
    fn check_field_access(&mut self, object: &Expr, field: &str) -> Result<VeltranoType, TypeCheckError> {
        let object_type = self.check_expression(object)?;
        
        // Look up field on the exact object type (no implicit conversions)
        self.get_field_type(&object_type, field)
            .ok_or_else(|| TypeCheckError::FieldNotFound {
                object_type: object_type.clone(),
                field: field.to_string(),
                location: object.location(),
            })
    }
    
    fn check_index_access(&mut self, object: &Expr, index: &Expr) -> Result<VeltranoType, TypeCheckError> {
        let object_type = self.check_expression(object)?;
        let index_type = self.check_expression(index)?;
        
        // Look up indexing on the exact object type (no implicit conversions)
        self.get_index_result_type(&object_type, &index_type)
            .ok_or_else(|| TypeCheckError::IndexingNotSupported {
                object_type: object_type.clone(),
                index_type,
                location: object.location(),
            })
    }
    
    fn check_binary_op(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<VeltranoType, TypeCheckError> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;
        
        // Look up operator on the exact operand types (no implicit conversions)
        self.get_binary_op_result_type(&left_type, op, &right_type)
            .ok_or_else(|| TypeCheckError::BinaryOperatorNotSupported {
                operator: op.clone(),
                left_type,
                right_type,
                location: left.location(),
            })
    }
    
    fn check_return(&mut self, expr: &Expr, expected_return: &VeltranoType) -> Result<(), TypeCheckError> {
        let actual_return = self.check_expression(expr)?;
        
        if !self.types_equal(expected_return, &actual_return) {
            return Err(TypeCheckError::TypeMismatch {
                expected: expected_return.clone(),
                actual: actual_return,
                location: expr.location(),
            });
        }
        
        Ok(())
    }
    
    // Core type equality check - no special deref handling
    fn types_equal(&self, a: &VeltranoType, b: &VeltranoType) -> bool {
        a == b  // Simple structural equality
    }
}
```

### Phase 3: Separate Error Analysis

When type checking fails, analyze the type mismatch and suggest fixes using simple, constant-time heuristics.

#### 3.1 Error Analyzer
```rust
pub struct ErrorAnalyzer;

impl ErrorAnalyzer {
    pub fn enhance_error(&self, error: TypeCheckError) -> TypeCheckError {
        match error {
            TypeCheckError::TypeMismatch { expected, actual, location } => {
                // Try to suggest a fix using simple pattern matching
                if let Some(suggestion) = self.suggest_conversion(&actual, &expected) {
                    TypeCheckError::TypeMismatchWithSuggestion {
                        expected,
                        actual,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::TypeMismatch { expected, actual, location }
                }
            }
            TypeCheckError::MethodNotFound { receiver_type, method, location } => {
                // Check if method exists on a deref'd version of the type
                if let Some(suggestion) = self.suggest_method_fix(&receiver_type, &method) {
                    TypeCheckError::MethodNotFoundWithSuggestion {
                        receiver_type,
                        method,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::MethodNotFound { receiver_type, method, location }
                }
            }
            TypeCheckError::FieldNotFound { object_type, field, location } => {
                // Check if field exists on a deref'd version of the type
                if let Some(suggestion) = self.suggest_field_fix(&object_type, &field) {
                    TypeCheckError::FieldNotFoundWithSuggestion {
                        object_type,
                        field,
                        location,
                        suggestion,
                    }
                } else {
                    TypeCheckError::FieldNotFound { object_type, field, location }
                }
            }
            _ => error,
        }
    }
    
    fn suggest_conversion(&self, from: &VeltranoType, to: &VeltranoType) -> Option<String> {
        // Simple constant-time pattern matching for common conversions
        match (from, to) {
            // Own<T> → T conversions
            (VeltranoType { base, ownership: Ownership::Owned, .. },
             VeltranoType { base: target_base, ownership: Ownership::Borrowed, .. }) 
            if base == target_base => {
                Some(".ref()".to_string())
            }
            
            // Own<String> → Str conversion (double deref)
            (VeltranoType { base: BaseType::String, ownership: Ownership::Owned, .. },
             VeltranoType { base: BaseType::Str, ownership: Ownership::Borrowed, .. }) => {
                Some(".ref().ref()".to_string())
            }
            
            // String → Str conversion  
            (VeltranoType { base: BaseType::String, ownership: Ownership::Borrowed, .. },
             VeltranoType { base: BaseType::Str, ownership: Ownership::Borrowed, .. }) => {
                Some(".ref()".to_string())
            }
            
            // Own<Vec<T>> → Vec<T> conversion
            (VeltranoType { base: BaseType::Vec(_), ownership: Ownership::Owned, .. },
             VeltranoType { base: BaseType::Vec(_), ownership: Ownership::Borrowed, .. }) => {
                Some(".ref()".to_string())
            }
            
            // Vec<T> → Slice<T> conversion
            (VeltranoType { base: BaseType::Vec(_), ownership: Ownership::Borrowed, .. },
             VeltranoType { base: BaseType::Slice(_), ownership: Ownership::Borrowed, .. }) => {
                Some(".toSlice()".to_string())
            }
            
            // MutRef<T> → T conversions
            (VeltranoType { base, ownership: Ownership::MutBorrowed, .. },
             VeltranoType { base: target_base, ownership: Ownership::Borrowed, .. })
            if base == target_base => {
                Some(".ref()".to_string())
            }
            
            // MutRef<String> → Str conversion (double conversion)
            (VeltranoType { base: BaseType::String, ownership: Ownership::MutBorrowed, .. },
             VeltranoType { base: BaseType::Str, ownership: Ownership::Borrowed, .. }) => {
                Some(".ref().ref()".to_string())
            }
            
            // Array to slice conversions: [T; N] → Slice<T>
            (VeltranoType { base: BaseType::Array(_, _), ownership: Ownership::Borrowed, .. },
             VeltranoType { base: BaseType::Slice(_), ownership: Ownership::Borrowed, .. }) => {
                Some(".toSlice()".to_string())
            }
            
            // Own<[T; N]> → Slice<T> conversion (needs ref first)
            (VeltranoType { base: BaseType::Array(_, _), ownership: Ownership::Owned, .. },
             VeltranoType { base: BaseType::Slice(_), ownership: Ownership::Borrowed, .. }) => {
                Some(".ref().toSlice()".to_string())
            }
            
            _ => None,
        }
    }
    
    fn suggest_method_fix(&self, receiver_type: &VeltranoType, method: &str) -> Option<String> {
        // Check if method would be available after .ref()
        match receiver_type.ownership {
            Ownership::Owned => {
                Some(format!("Try calling .ref().{method}() if the method exists on the borrowed type"))
            }
            _ => None,
        }
    }
    
    fn suggest_field_fix(&self, object_type: &VeltranoType, field: &str) -> Option<String> {
        // Check if field would be available after .ref()
        match object_type.ownership {
            Ownership::Owned => {
                Some(format!("Try accessing .ref().{field} if the field exists on the borrowed type"))
            }
            _ => None,
        }
    }
}
```

### Phase 4: Error Types and Messages

#### 4.1 Simplified Error Types
```rust
#[derive(Debug)]
pub enum TypeCheckError {
    TypeMismatch {
        expected: VeltranoType,
        actual: VeltranoType,
        location: SourceLocation,
    },
    TypeMismatchWithSuggestion {
        expected: VeltranoType,
        actual: VeltranoType,
        location: SourceLocation,
        suggestion: String,
    },
    MethodNotFound {
        receiver_type: VeltranoType,
        method: String,
        location: SourceLocation,
    },
    MethodNotFoundWithSuggestion {
        receiver_type: VeltranoType,
        method: String,
        location: SourceLocation,
        suggestion: String,
    },
    FieldNotFound {
        object_type: VeltranoType,
        field: String,
        location: SourceLocation,
    },
    FieldNotFoundWithSuggestion {
        object_type: VeltranoType,
        field: String,
        location: SourceLocation,
        suggestion: String,
    },
    ArgumentCountMismatch {
        function: String,
        expected: usize,
        actual: usize,
        location: SourceLocation,
    },
    IndexingNotSupported {
        object_type: VeltranoType,
        index_type: VeltranoType,
        location: SourceLocation,
    },
    BinaryOperatorNotSupported {
        operator: BinaryOp,
        left_type: VeltranoType,
        right_type: VeltranoType,
        location: SourceLocation,
    },
}
```

#### 4.2 Error Messages

##### Type Mismatch (Basic)
```
error: type mismatch
  --> examples/test.vl:3:25
   |
 3 |     val borrowed: String = owned
   |                            ^^^^^ expected `String`, found `Own<String>`
   |
   = note: Veltrano requires exact type matching
```

##### Type Mismatch (With Suggestion)
```
error: type mismatch
  --> examples/test.vl:3:25
   |
 3 |     val borrowed: String = owned
   |                            ^^^^^ expected `String`, found `Own<String>`
   |
   = help: try using `.ref()` to convert: `owned.ref()`
   = note: Veltrano requires explicit conversions for memory safety
```

##### Method Not Found (With Suggestion)  
```
error: method not found
  --> examples/test.vl:4:5
   |
 4 |     ownedString.length()
   |     ^^^^^^^^^^^ method `length` not found for type `Own<String>`
   |
   = help: try calling .ref().length() if the method exists on the borrowed type
   = note: methods are looked up on exact types only
```

##### Field Not Found (With Suggestion)
```
error: field not found
  --> examples/test.vl:6:18
   |
 6 |     val name = person.name
   |                ^^^^^^^^^^^ field `name` not found for type `Own<Person>`
   |
   = help: try accessing .ref().name if the field exists on the borrowed type
   = note: fields are looked up on exact types only
```

##### Function Argument Type Mismatch
```
error: type mismatch in function argument
  --> examples/test.vl:5:21
   |
 5 |     processString(ownedString)
   |                   ^^^^^^^^^^^ expected `Str`, found `Own<String>`
   |
   = help: try using `.ref().ref()` to convert: `ownedString.ref().ref()`
   = note: function `processString` expects parameter of type `Str`
```

#### 4.3 Error Message Formatting
```rust
impl TypeCheckError {
    pub fn format_message(&self) -> String {
        match self {
            TypeCheckError::TypeMismatch { expected, actual, location } => {
                format!(
                    "error: type mismatch\n  --> {}:{}:{}\n   |\n{:2} | {}\n   |{} expected `{}`, found `{}`\n   |\n   = note: Veltrano requires exact type matching",
                    location.file,
                    location.line,
                    location.column,
                    location.line,
                    location.source_line,
                    " ".repeat(location.column + 4),
                    expected,
                    actual
                )
            }
            
            TypeCheckError::TypeMismatchWithSuggestion { expected, actual, location, suggestion } => {
                format!(
                    "error: type mismatch\n  --> {}:{}:{}\n   |\n{:2} | {}\n   |{} expected `{}`, found `{}`\n   |\n   = help: {}\n   = note: Veltrano requires explicit conversions for memory safety",
                    location.file,
                    location.line,
                    location.column,
                    location.line,
                    location.source_line,
                    " ".repeat(location.column + 4),
                    expected,
                    actual,
                    suggestion
                )
            }
            
            TypeCheckError::MethodNotFound { receiver_type, method, location } => {
                format!(
                    "error: method not found\n  --> {}:{}:{}\n   |\n{:2} | {}\n   |{} method `{}` not found for type `{}`\n   |\n   = note: methods are looked up on exact types only",
                    location.file,
                    location.line,
                    location.column,
                    location.line,
                    location.source_line,
                    " ".repeat(location.column + 4),
                    method,
                    receiver_type
                )
            }
            
            TypeCheckError::MethodNotFoundWithSuggestion { receiver_type, method, location, suggestion } => {
                format!(
                    "error: method not found\n  --> {}:{}:{}\n   |\n{:2} | {}\n   |{} method `{}` not found for type `{}`\n   |\n   = help: {}\n   = note: methods are looked up on exact types only",
                    location.file,
                    location.line,
                    location.column,
                    location.line,
                    location.source_line,
                    " ".repeat(location.column + 4),
                    method,
                    receiver_type,
                    suggestion
                )
            }
            
            TypeCheckError::FieldNotFound { object_type, field, location } => {
                format!(
                    "error: field not found\n  --> {}:{}:{}\n   |\n{:2} | {}\n   |{} field `{}` not found for type `{}`\n   |\n   = note: fields are looked up on exact types only",
                    location.file,
                    location.line,
                    location.column,
                    location.line,
                    location.source_line,
                    " ".repeat(location.column + 4),
                    field,
                    object_type
                )
            }
            
            TypeCheckError::FieldNotFoundWithSuggestion { object_type, field, location, suggestion } => {
                format!(
                    "error: field not found\n  --> {}:{}:{}\n   |\n{:2} | {}\n   |{} field `{}` not found for type `{}`\n   |\n   = help: {}\n   = note: fields are looked up on exact types only",
                    location.file,
                    location.line,
                    location.column,
                    location.line,
                    location.source_line,
                    " ".repeat(location.column + 4),
                    field,
                    object_type,
                    suggestion
                )
            }
            
            // ... other error types
        }
    }
}
```

### Phase 4: Method Validation

#### 4.1 Explicit Conversion Methods
Ensure these methods are available and properly typed:

```veltrano
// For owned types
impl<T> Own<T> {
    fun ref(self): T        // Borrow the owned value
    fun clone(self): Own<T> // Clone the owned value
}

// For borrowed types  
impl<T> T {
    fun ref(self): Str      // Further borrowing (String → Str)
    fun own(self): Own<T>   // Take ownership (requires clone/copy)
    fun clone(self): T      // Clone the borrowed value
}
```

#### 4.2 Method Call Validation
```rust
impl VeltranoTypeChecker {
    fn check_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr]
    ) -> Result<VeltranoType, TypeCheckError> {
        let receiver_type = self.check_expression(receiver)?;
        
        match method {
            "ref" => self.check_ref_method(&receiver_type),
            "own" => self.check_own_method(&receiver_type),
            "clone" => self.check_clone_method(&receiver_type),
            _ => self.check_user_method(&receiver_type, method, args),
        }
    }
    
    fn check_ref_method(&self, receiver_type: &VeltranoType) -> Result<VeltranoType, TypeCheckError> {
        match &receiver_type.ownership {
            Ownership::Owned => {
                // Own<T> → T
                Ok(VeltranoType {
                    base: receiver_type.base.clone(),
                    ownership: Ownership::Borrowed,
                    mutability: receiver_type.mutability,
                })
            }
            Ownership::Borrowed => {
                // Handle String → Str conversion
                match &receiver_type.base {
                    BaseType::String => Ok(VeltranoType {
                        base: BaseType::Str,
                        ownership: Ownership::Borrowed,
                        mutability: receiver_type.mutability,
                    }),
                    _ => Err(TypeCheckError::InvalidMethod {
                        method: "ref".to_string(),
                        receiver_type: receiver_type.clone(),
                    }),
                }
            }
            _ => Err(TypeCheckError::InvalidMethod {
                method: "ref".to_string(),
                receiver_type: receiver_type.clone(),
            }),
        }
    }
}
```

## Testing Strategy

### Unit Tests for Strict Type Checking

#### Basic Type Mismatch Tests
```rust
#[test]
fn test_assignment_type_mismatch() {
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned  // Should error: type mismatch
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));
}

#[test]
fn test_assignment_with_suggestion() {
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned  // Should error with .ref() suggestion
    }
    "#;
    
    let result = parse_and_type_check(code);
    let enhanced_result = ErrorAnalyzer.enhance_error(result.unwrap_err());
    assert!(matches!(enhanced_result, TypeCheckError::TypeMismatchWithSuggestion { 
        suggestion, .. 
    } if suggestion.contains(".ref()")));
}

#[test]
fn test_assignment_exact_match_works() {
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val borrowed: String = owned.ref()    // Should work - exact type match
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(result.is_ok());
}
```

#### Function Argument Tests
```rust
#[test]
fn test_function_arg_type_mismatch() {
    let code = r#"
    fun processString(s: Str): Int {
        return s.length()
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val len = processString(owned)  // Should error: type mismatch
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(matches!(result, Err(TypeCheckError::TypeMismatch { .. })));
}

#[test]
fn test_function_arg_exact_match_works() {
    let code = r#"
    fun processString(s: Str): Int {
        return s.length()
    }
    
    fun main() {
        val owned: Own<String> = "hello".toString()
        val len = processString(owned.ref().ref())  // Should work - exact type match
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(result.is_ok());
}
```

#### Method Call Tests
```rust
#[test]
fn test_method_not_found() {
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val len = owned.length()  // Should error: method not found
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(matches!(result, Err(TypeCheckError::MethodNotFound { .. })));
}

#[test]
fn test_method_exact_receiver_works() {
    let code = r#"
    fun main() {
        val owned: Own<String> = "hello".toString()
        val len = owned.ref().length()  // Should work - exact receiver type
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(result.is_ok());
}
```

#### Field Access Tests  
```rust
#[test]
fn test_field_not_found() {
    let code = r#"
    data class Person(name: Own<String>, age: Int)
    
    fun main() {
        val person: Own<Person> = Person("Alice".toString(), 30)
        val name: Own<String> = person.name  // Should error: field not found on Own<Person>
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(matches!(result, Err(TypeCheckError::FieldNotFound { .. })));
}

#[test]
fn test_field_exact_object_works() {
    let code = r#"
    data class Person(name: Own<String>, age: Int)
    
    fun main() {
        val person: Own<Person> = Person("Alice".toString(), 30)
        val name: Own<String> = person.ref().name  // Should work - exact object type
    }
    "#;
    
    let result = parse_and_type_check(code);
    assert!(result.is_ok());
}
```

#### Error Analysis Tests
```rust
#[test]
fn test_error_analyzer_suggests_ref() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType { base: BaseType::String, ownership: Ownership::Borrowed, .. },
        actual: VeltranoType { base: BaseType::String, ownership: Ownership::Owned, .. },
        location: /* ... */,
    };
    
    let enhanced = ErrorAnalyzer.enhance_error(error);
    assert!(matches!(enhanced, TypeCheckError::TypeMismatchWithSuggestion { 
        suggestion, .. 
    } if suggestion.contains(".ref()")));
}

#[test]
fn test_error_analyzer_suggests_double_ref() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType { base: BaseType::Str, ownership: Ownership::Borrowed, .. },
        actual: VeltranoType { base: BaseType::String, ownership: Ownership::Owned, .. },
        location: /* ... */,
    };
    
    let enhanced = ErrorAnalyzer.enhance_error(error);
    assert!(matches!(enhanced, TypeCheckError::TypeMismatchWithSuggestion { 
        suggestion, .. 
    } if suggestion.contains(".ref().ref()")));
}

#[test]
fn test_error_analyzer_suggests_mut_ref_conversion() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType { base: BaseType::String, ownership: Ownership::Borrowed, .. },
        actual: VeltranoType { base: BaseType::String, ownership: Ownership::MutBorrowed, .. },
        location: /* ... */,
    };
    
    let enhanced = ErrorAnalyzer.enhance_error(error);
    assert!(matches!(enhanced, TypeCheckError::TypeMismatchWithSuggestion { 
        suggestion, .. 
    } if suggestion.contains(".ref()")));
}

#[test]
fn test_error_analyzer_suggests_array_to_slice() {
    let error = TypeCheckError::TypeMismatch {
        expected: VeltranoType { 
            base: BaseType::Slice(Box::new(VeltranoType { base: BaseType::Int, .. })), 
            ownership: Ownership::Borrowed, .. 
        },
        actual: VeltranoType { 
            base: BaseType::Array(Box::new(VeltranoType { base: BaseType::Int, .. }), 3), 
            ownership: Ownership::Borrowed, .. 
        },
        location: /* ... */,
    };
    
    let enhanced = ErrorAnalyzer.enhance_error(error);
    assert!(matches!(enhanced, TypeCheckError::TypeMismatchWithSuggestion { 
        suggestion, .. 
    } if suggestion.contains(".toSlice()")));
}
```

### Integration Tests

#### Simple Integration Test  
```veltrano
// Test file: tests/strict_type_checking.vl

// These should all fail with type mismatch errors:
fun testTypeMismatches() {
    // Assignment mismatches
    val ownedString: Own<String> = "test".toString()
    val borrowedString: String = ownedString  // ERROR: expected String, found Own<String>
    val strRef: Str = ownedString             // ERROR: expected Str, found Own<String>
    
    // Mutable reference mismatches
    val mutableRef: MutRef<String> = ownedString.mutRef()
    val immutableRef: String = mutableRef     // ERROR: expected String, found MutRef<String>
    
    // Array to slice mismatches
    val fixedArray: [Int; 3] = [1, 2, 3]
    val slice: Slice<Int> = fixedArray        // ERROR: expected Slice<Int>, found [Int; 3]
    
    // Function argument mismatches  
    processString(ownedString)                // ERROR: expected Str, found Own<String>
    processSlice(fixedArray)                  // ERROR: expected Slice<Int>, found [Int; 3]
    
    // Method not found
    val len = ownedString.length()            // ERROR: method length not found for Own<String>
}

fun processString(s: Str): Int { 
    return s.length() 
}

fun processSlice(items: Vec<Int>): Int {
    return items.size()
}

// These should all work with exact type matching:
fun testExactMatches() {
    val ownedString: Own<String> = "test".toString()
    val borrowedString: String = ownedString.ref()    // OK: String = String
    val strRef: Str = borrowedString.ref()            // OK: Str = Str
    
    // Mutable reference conversions
    val mutableRef: MutRef<String> = ownedString.mutRef()
    val immutableRef: String = mutableRef.ref()       // OK: explicit conversion
    
    // Array to slice conversions
    val fixedArray: [Int; 3] = [1, 2, 3]
    val slice: Slice<Int> = fixedArray.toSlice()      // OK: explicit conversion
    
    val len1 = processString(strRef)                  // OK: Str parameter
    val len2 = borrowedString.length()                // OK: method exists on String
    val size = processSlice(slice)                    // OK: Slice<Int> parameter
}
```

### Performance Tests
```rust
#[test]
fn test_type_checking_performance() {
    // Simple type checking should be very fast
    let large_code = generate_large_test_file(1000);
    
    let start = Instant::now();
    let result = parse_and_type_check(&large_code);
    let duration = start.elapsed();
    
    // Should be much faster without complex deref analysis
    assert!(duration < Duration::from_millis(50));
}

#[test]
fn test_error_analysis_performance() {
    // Error analysis should also be fast (constant time)
    let errors: Vec<TypeCheckError> = generate_type_mismatch_errors(1000);
    
    let start = Instant::now();
    let enhanced_errors: Vec<_> = errors.into_iter()
        .map(|e| ErrorAnalyzer.enhance_error(e))
        .collect();
    let duration = start.elapsed();
    
    // Constant-time pattern matching should be very fast
    assert!(duration < Duration::from_millis(10));
    assert_eq!(enhanced_errors.len(), 1000);
}
```

## Success Criteria

1. **Strict type checking enforced** - no implicit conversions allowed anywhere
2. **Simple and fast implementation** - type checker just does structural equality checks
3. **Helpful error suggestions** - constant-time analysis suggests `.ref()` fixes when applicable
4. **Explicit `.ref()` method calls work** for all supported conversions
5. **Excellent performance** - significantly faster than complex conversion analysis
6. **Clean separation of concerns** - type checking logic independent of error reporting
7. **Easy to maintain and extend** - simple codebase with clear responsibilities

## Migration Path for Existing Code

### Before (Current Behavior)
```veltrano
fun processText(input: Own<String>): Str {
    val text: String = input      // Implicit conversion
    val result: Str = text        // Implicit conversion  
    return result
}
```

### After (Required Explicit Conversions)
```veltrano
fun processText(input: Own<String>): Str {
    val text: String = input.ref()      // Explicit conversion
    val result: Str = text.ref()        // Explicit conversion
    return result
}
```

### Migration Tool (Future Enhancement)
```bash
# Automatically suggest explicit conversions
veltrano migrate --add-explicit-conversions examples/
```

## Implementation Timeline

1. **Phase 1** (2-3 days): Implement strict type checker with structural equality
2. **Phase 2** (1-2 days): Add simple error analyzer with constant-time suggestions  
3. **Phase 3** (1-2 days): Implement explicit conversion method validation
4. **Phase 4** (2-3 days): Add comprehensive tests and integration
5. **Phase 5** (1-2 days): Update existing examples to use explicit conversions

**Total estimated time: 1-2 weeks** (significantly faster than complex context-aware approach)

## Next Steps After Implementation

Once explicit type conversions are enforced, we can build upon this foundation to implement the remaining phases from [`type-system-design.md`](type-system-design.md):

1. **Phase 2: Lifetime Validation** - `@caller`, `@local` annotations and escape analysis
2. **Phase 3: Bump Allocation Constraints** - ensure `.bumpRef()` is only used in appropriate lifetime contexts  
3. **Phase 4: Data Class Validation** - ensure all required fields are provided in constructors
4. **Phase 5: Method Availability Checking** - validate which methods are available for each type

This Phase 1 implementation of enforcing explicit conversions establishes the **layered type checking** architecture and **explicit over implicit** principle that will guide all future Veltrano type system development.
