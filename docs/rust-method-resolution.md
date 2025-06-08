# Rust Method Resolution Algorithm

This document describes how Rust resolves method calls, which is critical for Veltrano's type system to correctly predict Rust behavior.

## Overview

When you write `receiver.method()` in Rust, the compiler must determine:
1. Which type implements the method
2. How to transform the receiver to match the method's `self` parameter
3. Which implementation to use when multiple options exist

## The Algorithm

### Step 1: Type Collection

The compiler starts with the receiver's type and builds a sequence of candidate types:

```rust
// Given: receiver: T
// Build sequence: T, &T, &mut T, &T, &&T, &mut &T, &&mut T, ...
```

For example, with `receiver: String`:
1. `String`
2. `&String` 
3. `&mut String`
4. `&&String`
5. `&mut &String`
6. ... (continuing with more references)

### Step 2: Method Search

For each type in the sequence, the compiler checks:

#### 2.1 Inherent Methods
```rust
impl String {
    fn custom_method(&self) { ... }  // Found here first
}
```

#### 2.2 Visible Trait Methods
```rust
impl Clone for String {
    fn clone(&self) -> Self { ... }  // Found if no inherent method
}
```

#### 2.3 Methods from Traits in Scope
Only traits that are imported or in the prelude are considered:
```rust
use std::fmt::Debug;  // Debug::fmt is now visible
// use std::fmt::Display;  // Display::fmt is NOT visible without this
```

### Step 3: Receiver Coercion

Once a method is found, Rust determines how to transform the receiver:

#### For `method(&self)`:
- `T` → `&T` (auto-ref)
- `&T` → `&T` (no change)
- `&&T` → `&&T` (no change)
- `&mut T` → `&T` (reborrow)

#### For `method(&mut self)`:
- `T` → `&mut T` (auto-mut-ref)
- `&mut T` → `&mut T` (no change)
- `&mut &T` → `&mut &T` (no change)

#### For `method(self)`:
- `T` → `T` (no change)
- `&T` → `&T` (no change)
- `&&T` → `&&T` (no change)

### Step 4: Deref Coercion

If no method is found, Rust follows the deref chain:

```rust
// For type &String
&String → String → str

// For type &&String  
&&String → &String → String → str

// For type Box<String>
Box<String> → String → str
```

After each deref, the process repeats from Step 2.

## Concrete Example: Clone

Let's trace through `(&&String).clone()`:

### Step 1: Build Candidate Types
Starting with `&&String`:
1. `&&String`
2. `&&&String`
3. `&mut &&String`
4. ... (more references)

### Step 2: Search for `clone` Method
For `&&String`:
- Inherent methods: None
- Trait methods: Check if `&&String` implements `Clone`
  - No direct impl
  - BUT: `impl<T> Clone for &T` matches with `T = &String`
  - This impl requires `&self`, so receiver must be `&(&String) = &&String` ✓

**Method found!** Use `<&T as Clone>::clone` where `T = &String`

### Result
- Input: `&&String`
- Uses: `impl<T> Clone for &T` with `T = &String`
- Output: `&String`

## Another Example: String Clone

Let's trace through `(&String).clone()`:

### Step 1: Build Candidate Types
Starting with `&String`:
1. `&String`
2. `&&String`
3. `&mut &String`
4. ... (more references)

### Step 2: Search for `clone` Method
For `&String`:
- Inherent methods: None
- Trait methods: Check if `&String` implements `Clone`
  - No impl for `&String`
  - No generic impl matches (would need `impl Clone for &String`)

### Step 3: Deref and Retry
Deref `&String` → `String`:
- Check if `String` implements `Clone`
  - Yes! `impl Clone for String`
  - Requires `&self`, receiver needs to be `&String` ✓

**Method found!** Use `<String as Clone>::clone`

### Result
- Input: `&String`
- Uses: `impl Clone for String`
- Output: `String`

## Key Rules

1. **Least Transformation Wins**: Rust prefers methods that require fewer transformations
2. **References Before Derefs**: Adding references is tried before dereferencing
3. **Trait Visibility Matters**: Traits must be in scope to have their methods considered
4. **Generic Implementations**: `impl<T> Trait for &T` can match reference types

## Implications for Veltrano

When translating Veltrano types to Rust behavior:

1. **Type Mapping**:
   - `String` in Veltrano → `&String` in Rust
   - `Ref<String>` in Veltrano → `&&String` in Rust
   - `Own<String>` in Veltrano → `String` in Rust

2. **Method Resolution**:
   - Must consider the actual Rust type (with references)
   - Check for implementations on reference types first
   - Only deref if no implementation found

3. **Clone Behavior**:
   - `String.clone()` → Uses `Clone for String` → Returns `Own<String>`
   - `Ref<String>.clone()` → Uses `Clone for &T` → Returns `String`

## Implementation Strategy

For correct method resolution in Veltrano:

1. Convert Veltrano type to Rust type (including natural references)
2. Check for methods on the exact type first
3. Consider generic implementations like `impl<T> Clone for &T`
4. Only strip references if no method found
5. Return the correct type based on which implementation was used

This ensures Veltrano's type system accurately predicts Rust's behavior.
