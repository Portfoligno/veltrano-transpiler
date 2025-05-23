# Veltrano Transpiler

A transpiler from Veltrano (Kotlin-like syntax) to Rust.

## Features

- Kotlin-like syntax with familiar keywords (`fun`, `var`, `val`)
- Type annotations and inference
- Control flow statements (`if`, `while`, `for`)
- Function declarations with parameters and return types
- Basic data types: `Int`, `String`, `Bool`, `Unit`

## Usage

```bash
cargo run <input.vl>
```

This will transpile the Veltrano source file to Rust and save it as `<input>.rs`.

## Example

**Input (hello.vl):**
```kotlin
fun main() {
    val message: String = "Hello, Veltrano!"
    println(message)
}
```

**Output (hello.rs):**
```rust
fn main() {
    let message: String = "Hello, Veltrano!";
    println(message);
}
```

## Language Syntax

### Variables
```kotlin
var mutable_var: Int = 42
val immutable_var: String = "Hello"
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

## Examples

See the `examples/` directory for sample Veltrano programs.