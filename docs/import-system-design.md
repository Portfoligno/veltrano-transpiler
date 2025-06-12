# Import System Design

## Overview

Veltrano's import system allows explicit importing of Rust methods for use in Veltrano code. The system is designed to be strict and predictable, with no implicit type conversions at the Veltrano level.

## Import Syntax

```veltrano
import Type.method              // Basic import
import Type.method as alias     // Import with alias
```

## Type Checking Rules

### Strict Type Matching

Imports are strictly type-checked based on the method's `self` parameter in Rust:

| Rust self kind | Required receiver for non-Copy types | Required receiver for Copy types |
|----------------|-------------------------------------|----------------------------------|
| `Self` (by value) | `Own<Self>` | Bare `Self` |
| `&Self` | Bare `Self` | `Ref<Self>` |
| `&mut Self` | `MutRef<Self>` | `MutRef<Ref<Self>>` |
| No self (static) | N/A - called as function | N/A - called as function |

### Key Principle: No Autoref at Veltrano Level

When you import a method, it can ONLY be used on the exact Veltrano type specified in the import:

- `import String.clone` → Only usable on Veltrano type `String`
- `import Vec.push` → Only usable on Veltrano type `Vec`

This is true even when the Rust method takes `&self` - the Veltrano receiver must be exactly the imported type, not a reference to it.

## Type Transpilation

Understanding how Veltrano types transpile to Rust is crucial for imports:

| Veltrano Type | Rust Type |
|---------------|-----------|
| `String` | `&String` |
| `Ref<String>` | `&&String` |
| `Own<String>` | `String` |
| `Vec` | `&Vec<T>` |
| `Own<Vec>` | `Vec<T>` |

## Examples

### Example 1: String.clone

```veltrano
import String.clone

fun example(s: Own<String>) {
    val s1: String = s.ref()      // s1 has type String
    val s2 = s1.clone()           // OK: calling clone on String
    
    val s3: Ref<String> = s1.ref() // s3 has type Ref<String>
    val s4 = s3.clone()           // ERROR: import String.clone doesn't work on Ref<String>
}
```

The transpiled Rust code for the successful case:
```rust
let s2 = String::clone(&s1);  // s1 is &String in Rust
```

### Example 2: Multiple Imports with Same Name

```veltrano
import String.clone as duplicate
import i64.abs as duplicate

fun example() {
    val text: String = "hello"
    val num: i64 = -42
    
    val text_copy = text.duplicate()  // Resolves to String.clone
    val positive = num.duplicate()    // Resolves to i64.abs
}
```

### Example 3: Import vs Built-in Methods

```veltrano
import String.len  // This completely shadows any built-in len method

fun example(s: String) {
    val length = s.len()  // Uses imported String.len, not built-in
}
```

## Design Rationale

### Why Strict Type Matching?

1. **Predictability**: You know exactly which type can use an imported method
2. **Explicitness**: No hidden autoref/autoderef conversions at the Veltrano level
3. **Clarity**: The import clearly states which Veltrano type it applies to
4. **Safety**: Prevents confusion about which trait implementation is being used

### Comparison with Built-in Methods

Built-in methods in Veltrano support autoref/autoderef for convenience. Imports do not, ensuring that:
- The exact trait implementation is always clear
- There's no ambiguity about which method is called
- The transpiled Rust code is predictable

## Implementation Notes

### Type Checking

The type checker enforces strict matching:
```rust
// For methods expecting &self
SelfKind::Ref => {
    // Receiver must be exactly the imported type
    receiver_type == &import_veltrano_type
}
```

### Code Generation

The code generator produces UFCS calls:
```rust
// Veltrano: text.clone() where text: String
// Rust: String::clone(&text) where text: &String
```

The Rust compiler handles the necessary referencing based on the method signature.
