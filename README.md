# Veltrano Transpiler

A transpiler from Veltrano (Kotlin-like syntax) to Rust with a reference-by-default type system.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Overview](#overview)
- [Type System](#type-system)
- [Language Guide](#language-guide)
- [Command Line Reference](#command-line-reference)
- [Examples](#examples)
- [Design Decisions](#design-decisions)

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/veltrano-transpiler.git
cd veltrano-transpiler

# Build the project
cargo build --release

# Run tests
cargo test
```

## Quick Start

```bash
# Transpile a Veltrano file to Rust
cargo run examples/hello.vl

# With comment preservation
cargo run -- --preserve-comments examples/hello.vl

# The output will be saved as hello.rs
```

**Example Input (hello.vl):**
```kotlin
fun main() {
    val message: Str = "Hello, Veltrano!"
    println("{}", message)
}
```

**Generated Output (hello.rs):**
```rust
fn main() {
    let message: &str = "Hello, Veltrano!";
    println!("{}", message);
}
```

## Overview

### Design Philosophy

Veltrano is designed with these core principles:

1. **Reference-by-default**: Borrowing is the common case in Rust, so Veltrano makes it the default
2. **Explicit ownership**: When you need ownership, use `Own<T>` to make it visible and intentional
3. **Kotlin-like syntax**: Familiar syntax for Kotlin developers with Rust's powerful semantics
4. **Immutability-first**: Only `val` (immutable bindings) are supported, encouraging functional patterns

### Key Features

- **Familiar Syntax**: Kotlin-like keywords (`fun`, `val`, `if`, `while`)
- **Type Safety**: Full type annotations and inference
- **Automatic Conversions**: camelCase to snake_case naming
- **Comment Preservation**: Optional comment preservation in generated code
- **Clear Semantics**: Direct mapping to idiomatic Rust patterns

## Type System

### Core Concepts

In Veltrano, the type system is built around making references the default:

- Types without `Own<>` are **references by default**
- Use `Own<T>` for **explicit ownership**
- Basic types (`Int`, `Bool`, `Unit`, `Nothing`) are **always owned** (matching Rust's Copy types)
- Use `Ref<T>` for **additional reference levels**
- Use `MutRef<T>` for **mutable references**

### Type Mapping Reference

| Veltrano Type | Rust Type | Description | Example |
|---------------|-----------|-------------|---------|
| **Basic Types** |
| `Int` | `i64` | 64-bit integer (always owned) | `val x: Int = 42` |
| `Bool` | `bool` | Boolean (always owned) | `val flag: Bool = true` |
| `Unit` | `()` | Unit type | `fun doSomething(): Unit` |
| `Nothing` | `!` | Never type | `fun abort(): Nothing` |
| **String Types** |
| `Str` | `&str` | String slice reference | `val s: Str = "hello"` |
| `String` | `&String` | String reference | `val s: String = owned.ref()` |
| `Own<String>` | `String` | Owned string | `val s: Own<String> = "hello".toString()` |
| `Box<Str>` | `Box<str>` | Boxed string slice | `val s: Box<Str> = "hello".into()` |
| **Reference Types** |
| `T` | `&T` | Reference to T (default) | `val x: String = owned.ref()` |
| `Own<T>` | `T` | Owned T | `val x: Own<String> = "hello".toString()` |
| `Ref<T>` | `&&T` | Additional reference level | `val x: Ref<String> = s.ref()` |
| `MutRef<T>` | `&mut &T` | Mutable reference | `val x: MutRef<String> = MutRef(borrowed)` |

### Working with References

#### The `.ref()` Method

Convert owned values to references:

```kotlin
val owned: Own<String> = "Hello".toString()  // String (owned)
val borrowed: String = owned.ref()            // &String (reference)
val doubleBorrowed: Ref<String> = borrowed.ref()  // &&String
```

#### The `MutRef()` Function and `.mutRef()` Method

Create mutable references with two available syntaxes:

```kotlin
// Preferred: MutRef() function - generates &mut (&value).clone()
val number: Int = 42
val mutableRef: MutRef<Int> = MutRef(number)

// Alternative: .mutRef() method - generates &mut value
// Chain directly without binding to avoid immutability issues
val mutableRef2: MutRef<Int> = number.ref().clone().mutRef()
```

**Example with function:**
```kotlin
fun modify(value: MutRef<Int>) {
    // Function accepting a mutable reference
}

fun main() {
    val number: Int = 42
    val mutableRef: MutRef<Int> = MutRef(number)
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
    let mutable_ref: &mut i64 = &mut (&number).clone();
    modify(mutable_ref);
}
```

The `MutRef()` function automatically handles the borrow-and-clone pattern, making it the preferred approach for creating mutable references in Veltrano's immutability-first design.

## Language Guide

### Variables

Veltrano uses `val` for immutable variable bindings:

```kotlin
val name: Str = "Alice"             // String slice (explicitly typed)
val age = 25                        // Type inference (Int)
val message = "Hello, World!"       // Type inference (Str)
val owned: Own<String> = "Bob".toString()  // Owned string
```

### Functions

Functions are declared with `fun`:

```kotlin
// Basic function
fun greet(name: String) {
    println("Hello, {}!", name)
}

// With return type
fun add(a: Int, b: Int): Int {
    return a + b
}

// Expression body (explicit return needed)
fun multiply(x: Int, y: Int): Int {
    return x * y
}
```

### Control Flow

#### If Statements

```kotlin
if (x > 0) {
    println("positive")
} else if (x < 0) {
    println("negative")
} else {
    println("zero")
}
```

#### While Loops

```kotlin
val counter = 0
while (counter < 10) {
    println("{}", counter)
    // counter would need to be mutable to increment
}

// Infinite loops are converted to Rust's loop
while (true) {
    // Becomes: loop { ... }
}
```

### Comments

Both line and block comments are supported:

```kotlin
// This is a line comment

/* This is a
   block comment */

val x = 42  // Inline comment

/* 
 * Documentation-style
 * block comment
 */
fun documented() {
    // Implementation
}
```

### Never Type

The `Nothing` type represents computations that never return:

```kotlin
fun abort(message: Str): Nothing {
    panic("{}", message)  // Never returns
}

fun infiniteLoop(): Nothing {
    while (true) {
        // Never returns - transpiles to Rust's loop
    }
}
```

### Naming Conventions

Veltrano automatically converts Kotlin's camelCase to Rust's snake_case:

| Veltrano (Input) | Rust (Output) |
|-----------------|---------------|
| `calculateSum` | `calculate_sum` |
| `firstName` | `first_name` |
| `HTTPSConnection` | `h_t_t_p_s_connection` |
| `getValue` | `get_value` |

This applies to all identifiers: functions, variables, and parameters.

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

## Command Line Reference

### Basic Usage

```bash
cargo run [OPTIONS] <input-file>
```

### Options

| Option | Description |
|--------|-------------|
| `--preserve-comments` | Include comments from source in generated Rust code |

### Examples

```bash
# Basic transpilation
cargo run examples/hello.vl
# Output: examples/hello.rs

# With preserved comments
cargo run -- --preserve-comments examples/fibonacci.vl
# Output: examples/fibonacci.rs (with comments)

# From any directory
cargo run path/to/myfile.vl
# Output: path/to/myfile.rs
```

## Examples

The `examples/` directory contains various Veltrano programs demonstrating different features:

- `hello.vl` - Basic hello world program
- `fibonacci.vl` - Fibonacci sequence calculation
- `string_types.vl` - Different string type usage
- `ref_type.vl` - Reference type demonstrations
- `mut_ref.vl` - Mutable reference usage
- `type_symmetry.vl` - Type system symmetry examples
- `unified_types.vl` - Unified type system examples
- `never_type.vl` - Never type (`Nothing`) usage
- `camel_case.vl` - Naming convention examples
- `comments.vl` - Comment preservation examples
- `clone_ufcs.vl` - UFCS clone behavior
- `mutable_bindings.vl` - Practical MutRef patterns
- `mutref_syntax_comparison.vl` - Comparison of MutRef syntaxes

## Design Decisions

### Why No `var` (For Now)

Veltrano currently supports only `val` (immutable variables) without a `var` keyword. This is a deliberate choice based on a semantic mismatch between Kotlin's `var` and Rust's `let mut`.

#### The Semantic Challenge

In Kotlin, `var` is a **storage declaration** - it only affects whether the variable binding can be reassigned:

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

This fundamental difference makes a direct mapping potentially confusing. For now, Veltrano takes an immutability-first approach, using `MutRef<T>` for cases where mutation is needed.

### Reference-by-Default Design

Most types in Veltrano are references by default because:

1. **Borrowing is more common** than ownership in typical Rust code
2. **Explicit ownership** with `Own<T>` makes ownership transfers visible
3. **Prevents accidental moves** which are a common source of Rust learner confusion
4. **Aligns with Rust's philosophy** of preferring borrowing

This design choice means:
- `String` in Veltrano is `&String` in Rust (not `String`)
- To get an owned string, use `Own<String>`
- Basic types like `Int` and `Bool` remain owned (matching Rust's Copy types)

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Setup

```bash
# Clone and build
git clone https://github.com/yourusername/veltrano-transpiler.git
cd veltrano-transpiler
cargo build

# Run tests
cargo test

# Run with example
cargo run examples/hello.vl
```

## License

[Specify your license here]