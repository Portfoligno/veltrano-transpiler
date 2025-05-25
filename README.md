# Veltrano Transpiler

A transpiler from Veltrano (Kotlin-like syntax) to Rust.

## Features

- Kotlin-like syntax with familiar keywords (`fun`, `val`)
- Type annotations and inference
- Control flow statements (`if`, `while`, `for`)
- Function declarations with parameters and return types
- Basic data types: `Int`, `Bool`, `Unit`, `Nothing`
- String types: `Ref<Str>`, `String`, `Ref<String>`, `Box<Str>`
- Comments: Line comments (`//`) and block comments (`/* */`)

## Usage

```bash
cargo run <input.vl>
```

This will transpile the Veltrano source file to Rust and save it as `<input>.rs`.

## Example

**Input (hello.vl):**
```kotlin
fun main() {
    val message: Ref<Str> = "Hello, Veltrano!"
    println("{}", message)
}
```

**Output (hello.rs):**
```rust
fn main() {
    let message: &str = "Hello, Veltrano!";
    println!("{}", message);
}
```

## Language Syntax

### Variables
```kotlin
val immutable_var: Ref<Str> = "Hello"
```

### Functions
```kotlin
fun add(a: Int, b: Int): Int {
    return a + b
}
```

### Control Flow
```kotlin
if (condition) {
    // then branch
} else {
    // else branch
}

while (condition) {
    // loop body
}
```

### String Types

Veltrano provides precise string type control that maps to Rust's string types.

**Note on `Ref<T>` naming:** In Veltrano, `Ref<T>` represents a borrowed reference and transpiles to Rust's `&T`. While this name conflicts with Rust's `std::cell::Ref`, it was chosen for consistency with common Rust idioms like `.as_ref()` and the `ref` keyword. Rust developers are familiar with context-dependent meanings of "ref" throughout the standard library.

| Veltrano Type | Rust Type | Description |
|---------------|-----------|-------------|
| `Ref<Str>` | `&str` | String slice, immutable reference |
| `String` | `String` | Owned, growable string |
| `Ref<String>` | `&String` | Reference to owned string |
| `Box<Str>` | `Box<str>` | Owned, fixed-size string |

### Never Type

| Veltrano Type | Rust Type | Description |
|---------------|-----------|-------------|
| `Nothing` | `!` | Never type - functions that never return |

**String Examples:**
```kotlin
val literal: Ref<Str> = "Hello"           // &str (string literals are already references)
val owned: String = "Hello".toString()    // String
val borrowed: Ref<String> = owned.ref()   // &String (taking reference with .ref() method)
val boxed: Box<Str> = "Hello".into()      // Box<str>
```

**Never Type Examples:**
```kotlin
fun abort(message: Ref<Str>): Nothing {
    panic("{}", message)  // Never returns
}

fun infiniteLoop(): Nothing {
    while (true) {
        // Never returns
    }
}
```

**Transpiles to:**
```rust
fn abort(message: &str) -> ! {
    panic!("{}", message);  // Never returns
}

fn infinite_loop() -> ! {
    loop {
        // Never returns
    }
}
```

### Reference Creation with `.ref()`

Veltrano provides a convenient `.ref()` method to create references, which transpiles to Rust's `&` operator:

```kotlin
val owned: String = "Hello".toString()
val borrowed: Ref<String> = owned.ref()  // Becomes &owned in Rust
```

**When to use `.ref()`:**
- Taking references of owned values: `owned.ref()` → `&owned`
- Creating `Ref<String>` from `String`
- Creating `Ref<CustomType>` from `CustomType`

**When NOT to use `.ref()`:**
- String literals are already references: `"hello"` is already `Ref<Str>` (`&str`)
- Values that are already reference types

### Naming Convention Conversion

Veltrano uses Kotlin's camelCase naming convention, which is automatically converted to Rust's snake_case convention during transpilation:

| Veltrano (camelCase) | Rust (snake_case) |
|---------------------|-------------------|
| `calculateSum` | `calculate_sum` |
| `firstName` | `first_name` |
| `veryLongVariableName` | `very_long_variable_name` |
| `XMLParser` | `x_m_l_parser` |
| `httpURLConnection` | `http_u_r_l_connection` |
| `a` | `a` |
| `aB` | `a_b` |

This conversion applies to:
- Function names
- Variable names  
- Parameter names
- All identifier references

**Example:**
```kotlin
fun calculateSum(firstNumber: Int, secondNumber: Int): Int {
    val totalResult: Int = firstNumber + secondNumber
    return totalResult
}
```

**Transpiles to:**
```rust
fn calculate_sum(first_number: i64, second_number: i64) -> i64 {
    let total_result: i64 = first_number + second_number;
    return total_result;
}
```

### Comments

Veltrano supports both line comments and block comments, similar to many C-style languages:

```kotlin
// This is a line comment
fun main() {
    // Variable declaration with comment
    val message: Ref<Str> = "Hello, World!"
    
    /* This is a
       multi-line block comment
       that spans several lines */
    println("{}", message)
    
    // Another line comment
    val number: Int = 42 // Inline comment
    println("{}", number)
}
```

**Transpiles to:**
```rust
// This is a line comment
fn main() {
    // Variable declaration with comment
    let message: &str = "Hello, World!";
    
    /* This is a
       multi-line block comment
       that spans several lines */
    println!("{}", message);
    
    // Another line comment
    let number: i64 = 42; // Inline comment
    println!("{}", number);
}
```

Comments are preserved during transpilation with their original formatting intact.

## Examples

See the `examples/` directory for sample Veltrano programs.

## Design Notes

### Why No `var` (For Now)

Veltrano currently supports only `val` (immutable variables) without a `var` keyword. This is a deliberate choice based on a semantic mismatch between Kotlin's `var` and Rust's `let mut` that I haven't yet found a satisfactory way to resolve.

#### The Semantic Challenge

In Kotlin, `var` is a **storage declaration** that only affects whether the variable binding can be reassigned:

```kotlin
// Kotlin
var x = 5
x = 10  // OK - rebinding allowed

val list = mutableListOf(1, 2, 3)
list.add(4)  // OK - val doesn't prevent mutation of the data!
```

In Rust, `let mut` has **recursive semantics** - it affects both the binding AND enables mutation of the data:

```rust
// Rust
let mut x = vec![1, 2, 3];
x.push(4);      // Mutating the data (requires mut)
x = vec![5, 6]; // Rebinding (also requires mut)

let y = vec![1, 2, 3];
y.push(4);      // ERROR - cannot mutate without mut
```

This fundamental difference makes a direct mapping of `var` to `let mut` potentially confusing. Kotlin developers would expect `var` to only control rebinding, not data mutability.

#### Current Approach

While exploring syntax options that accurately represent both languages' semantics, Veltrano takes an immutability-first approach:

1. **Direct Mapping** - `val` corresponds directly to `let`, maintaining clear semantics

2. **Consistent Behavior** - No surprising differences between source and target language behavior

3. **Functional Patterns** - Encourages immutable data transformations

4. **Future Flexibility** - Leaves room for a more nuanced mutability syntax that accurately represents both Kotlin and Rust semantics

This is an active area of language design for Veltrano. I'm exploring ways to support mutability that feel natural to Kotlin developers while generating idiomatic Rust code.
