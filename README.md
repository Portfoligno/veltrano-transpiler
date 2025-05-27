# Veltrano Transpiler

A transpiler from Veltrano (Kotlin-like syntax) to Rust.

## Features

**Language Syntax:**
- Kotlin-like syntax with familiar keywords (`fun`, `val`)
- Type annotations and inference
- Control flow statements (`if`, `while`, `for`)
- Function declarations with parameters and return types
- Comments: Line comments (`//`) and block comments (`/* */`)

**Type System:**
- Reference-by-default design: `String`, `Str` are references by default
- Explicit ownership: `Own<T>` for owned values
- Explicit references: `Ref<T>` for additional reference levels
- Mutable references: `MutRef<T>` with `MutRef()` function and `.mutRef()` method
- Basic types: `Int`, `Bool`, `Unit`, `Nothing` (always owned)

## Usage

```bash
cargo run <input.vl>
```

This will transpile the Veltrano source file to Rust and save it as `<input>.rs`.

### Options

- `--preserve-comments` - Preserve comments from the source file in the generated Rust code

```bash
cargo run -- --preserve-comments <input.vl>
```

## Example

**Input (hello.vl):**
```kotlin
fun main() {
    val message: Str = "Hello, Veltrano!"
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
val immutable_var: Str = "Hello"
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

**Note on reference-by-default design:** In Veltrano, types without `Own<>` are references by default. This design treats borrowing as the default case and makes ownership explicit when needed. Types like `Int` and `Bool` are always owned.

**Own<T> restrictions:** Cannot wrap already-owned types. Valid: `Own<String>`, `Own<Str>`, `Own<CustomType>`.

| Veltrano Type | Rust Type | Description |
|---------------|-----------|-------------|
| `Str` | `&str` | String slice reference |
| `String` | `&String` | String reference |
| `Own<String>` | `String` | Owned string |
| `Box<Str>` | `Box<str>` | Owned, fixed-size string |

### Never Type

| Veltrano Type | Rust Type | Description |
|---------------|-----------|-------------|
| `Nothing` | `!` | Never type - functions that never return |

**String Examples:**
```kotlin
val literal: Str = "Hello"                   // &str
val owned: Own<String> = "Hello".toString()  // String (owned)
val borrowed: String = owned.ref()           // &String (reference to owned)
val boxed: Box<Str> = "Hello".into()         // Box<str>
```

**Never Type Examples:**
```kotlin
fun abort(message: Str): Nothing {
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
val owned: Own<String> = "Hello".toString()
val borrowed: String = owned.ref()  // Becomes &owned in Rust
```

**When to use `.ref()`:**
- Converting owned values to references: `Own<T>` → `T`
- Example: `Own<String>` → `String` (which transpiles to `&String`)

**Remember:**
- Types without `Own<>` are already references
- `.ref()` is only needed when you have an owned value (`Own<T>`) and need a reference

### Mutable References with `MutRef<T>` and `MutRef()` function

Veltrano supports mutable references through the `MutRef<T>` type and `MutRef()` function:

| Veltrano Type | Rust Type | Description |
|---------------|-----------|-------------|
| `MutRef<T>` | `&mut T` | Mutable reference to type T |

```kotlin
fun modify(value: MutRef<Int>) {
    // Function accepting a mutable reference
}

fun main() {
    val number: Int = 42
    val mutableRef: MutRef<Int> = MutRef(number)  // MutRef() creates a mutable reference to a clone
    modify(mutableRef)
}
```

**Transpiles to:**
```rust
fn modify(value: &mut i64) {
    // Function accepting a mutable reference
}

fn main() {
    let number: i64 = 42;
    let mutable_ref: &mut i64 = &mut (&number).clone();  // MutRef() creates a mutable reference to a clone
    modify(mutable_ref);
}
```

**Note:** The `MutRef()` function automatically handles borrowing and cloning, generating `&mut (&value).clone()` in Rust. This provides a clean syntax while making the cloning explicit and aligning with Rust's ownership principles. The `.mutRef()` method is also available (generating `&mut value`) for symmetry with `.ref()`, but `MutRef()` is the preferred approach.

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
    val message: Str = "Hello, World!"
    
    /* This is a
       multi-line block comment
       that spans several lines */
    println("{}", message)
    
    // Another line comment
    val number: Int = 42 // Inline comment
    println("{}", number)
}
```

By default, comments are not included in the generated Rust code. To preserve comments during transpilation, use the `--preserve-comments` flag:

```bash
cargo run -- --preserve-comments input.vl
```

**With `--preserve-comments`, transpiles to:**
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
