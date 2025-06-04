# Lifetime Labels Design Draft

## Overview

This document proposes a Kotlin-inspired label syntax for explicit lifetime parameters in Veltrano. Each lifetime label has its own bump allocator that is passed implicitly when references with that lifetime are created.

## Core Concepts

### 1. Lifetime Labels
- Lifetime parameters are declared using `@` prefix: `@a`, `@b`, `@caller`
- Each function name automatically serves as a lifetime label
- Lifetimes are declared in angle brackets: `fun<@a, @b> functionName(...)`

### 2. Lifetime Annotations on Types
- Types can be annotated with lifetimes: `Ref<Str@a>`, `User@a`
- Data classes can have lifetime parameters: `data class User<@a>(...)`
- Without annotation, references use the current function's lifetime

### 3. Bump Allocator Association
- Each lifetime label has an associated bump allocator
- When `.bumpRef()` is called, it uses the allocator for the target lifetime
- Allocators are passed implicitly based on lifetime usage

## Syntax Examples

### Basic Lifetime Parameter
```kotlin
fun<@a> createRef(value: Str): Ref<Str@a> {
    return value.bumpRef()  // Uses @a's bump allocator
}
```

### Multiple Lifetime Parameters
```kotlin
fun<@a, @b> selectFirst(ref1: Ref<Str@a>, ref2: Ref<Str@b>): Ref<Str@a> {
    if (someCondition()) {
        return ref1
    } else {
        return ref2.clone().bumpRef()  // Clone into @a's lifetime
    }
}
```

### Data Classes with Lifetimes
```kotlin
data class Container<@a>(
    data: Ref<Str@a>,
    count: Int
)

// Usage
fun<@x> makeContainer(s: Str): Container@x {
    return Container(data = s.bumpRef(), count = 1)
}
```

### Function's Own Lifetime
```kotlin
fun processLocally(items: Array<Str>): Summary {
    // Implicit @processLocally lifetime available
    val tempRefs = items.map(|s| s.bumpRef())  // Uses @processLocally
    
    // tempRefs cannot escape - they're bound to @processLocally
    return Summary(count = tempRefs.length)  // OK - returns owned data
}
```

## Lifetime Rules

### 1. Escape Prevention
References with a function's own lifetime cannot escape that function:
```kotlin
fun badExample(): Ref<Str@badExample> {  // ERROR: Cannot return @badExample
    return "hello".bumpRef()
}
```

### 2. Lifetime Compatibility
References can only be assigned/passed where lifetimes match:
```kotlin
fun<@a, @b> example(ref1: Ref<Str@a>) {
    val ref2: Ref<Str@b> = ref1  // ERROR: Lifetime mismatch
    val ref3: Ref<Str@a> = ref1  // OK
}
```

### 3. Implicit Function Lifetime
Every function has an implicit lifetime with its own name:
```kotlin
fun helper(s: Str) {
    val local = s.bumpRef()  // Implicitly Ref<Str@helper>
    useLocally(local)        // OK - used within helper
}  // local's memory cleaned up here
```

### 4. Nested Reference Lifetimes and Inference
Since `Ref<Str>` translates to `&&str` in Rust, there are two distinct reference levels:

```kotlin
// Explicit form showing both lifetime levels
val ref: Ref@a<Str@b> = ...  // Maps to &'a &'b str

// When lifetimes are omitted, both default to the same inferred lifetime
val ref: Ref<Str> = ...      // Maps to &'inferred &'inferred str

// In function context, both would use function's lifetime
fun helper(s: Str) {
    val local: Ref<Str> = s.bumpRef()  // Ref@helper<Str@helper> -> &'helper &'helper str
}
```

**Inference Rule**: When lifetime parameters are omitted from nested reference types, all reference levels use the same inferred lifetime. The inference context (function scope, data class parameter, etc.) determines what that lifetime is.

**Explicit Control**: Developers can specify different lifetimes for each reference level when needed:
```kotlin
fun<@outer, @inner> complexRef(): Ref@outer<Str@inner> {
    // Outer reference lives in @outer, inner reference lives in @inner
    // Maps to &'outer &'inner str
}
```

## Code Generation

### Bump Allocator Passing
Functions with lifetime parameters receive hidden bump allocator parameters:
```kotlin
// Veltrano
fun<@a> createRef(value: Str): Ref<Str@a> {
    return value.bumpRef()
}
```
```rust
// Generated Rust
fn createRef<'a>(value: &str, __bump_a: &'a Bump) -> &'a str {
    __bump_a.alloc_str(value)
}
```

### Multiple Lifetimes
```kotlin
// Veltrano
fun<@a, @b> process(x: Container@a, y: Container@b): Container@a {
    // ...
}
```
```rust
// Generated Rust
fn process<'a, 'b>(
    x: Container<'a>, 
    y: Container<'b>,
    __bump_a: &'a Bump,
    __bump_b: &'b Bump
) -> Container<'a> {
    // ...
}
```

## Migration Strategy

### Phase 1: Add Syntax Support
- Extend AST to support lifetime parameters
- Update parser to handle `@lifetime` syntax
- Keep existing implicit bump allocation for compatibility

### Phase 2: Lifetime Analysis
- Implement lifetime checking and validation
- Generate appropriate Rust lifetime parameters
- Pass bump allocators based on lifetime usage

### Phase 3: Deprecate Implicit System
- Encourage explicit lifetime annotations
- Provide migration tooling
- Eventually remove automatic bump detection

## Benefits

1. **Explicit Control**: Developers can control which allocator is used
2. **Safety**: Compile-time prevention of lifetime escapes
3. **Performance**: Scoped allocators can be freed when no longer needed
4. **Clarity**: Lifetime relationships are visible in function signatures

## Open Questions

1. Should we allow lifetime elision in simple cases?
2. How to handle lifetime inference for local variables?
3. Should data class fields require matching lifetimes?
4. How to integrate with existing reference depth system?
