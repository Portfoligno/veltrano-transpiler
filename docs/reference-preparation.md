# Cross-Scope Reference Preparation

## Overview

The automatic bump parameter detection feature enables functions to prepare references that remain valid in the caller's scope. This allows functions to build and return reference-containing data structures without requiring the caller to manage memory allocation details.

## Problem Statement

In systems programming, it's common to need functions that:
1. Build complex data structures containing references
2. Return these structures to the caller for further use
3. Ensure references remain valid after the function returns

Without cross-scope reference preparation, this requires either:
- Manual lifetime parameter threading through all functions
- Copying data instead of using references (performance cost)
- Complex ownership patterns that limit API design

## Current Implementation

**Status:** Prototype implementation using shared bump allocator

The current implementation uses a single bump allocator instance that:
- Is automatically passed to functions that need to prepare references
- Keeps all allocated references alive for the entire program duration
- Eliminates lifetime complexity by essentially "leaking" everything
- Serves as a placeholder until proper lifetime detection is implemented

**Automatic Detection:** The transpiler analyzes function call graphs to determine which functions need access to the shared bump allocator, eliminating manual parameter threading.

## Intended Use Cases

### 1. Data Structure Preparation
Functions that build reference-containing data structures for caller use:

```veltrano
fun buildConfiguration(app_name: Str, version: Str): Config {
    // Prepare references that will be valid in caller's scope
    val name_ref = app_name.bumpRef()
    val version_ref = version.bumpRef()
    return Config(name = name_ref, version = version_ref)
}

// Caller can use the returned structure with valid references
val config = buildConfiguration("MyApp", "1.0.0")
```

### 2. Factory Functions
Functions that create and configure complex objects with references:

```veltrano
fun createUserSession(email: Str, permissions: Array<Str>): Session {
    val user = User(email = email.bumpRef())
    val perm_refs = permissions.map(|p| p.bumpRef())
    return Session(user = user, permissions = perm_refs)
}
```

### 3. Parser/Builder Patterns
Functions that parse input and build structured data with references:

```veltrano
fun parseConfiguration(input: Str): ParsedConfig {
    val sections = input.split("\n")
    val section_refs = sections.map(|s| s.bumpRef())
    return ParsedConfig(sections = section_refs)
}
```

### 4. View/Presentation Layer
Functions that prepare data views containing references for UI rendering:

```veltrano
fun prepareUserDisplay(user: User, messages: Array<Message>): UserView {
    val display_name = formatName(user.name).bumpRef()
    val message_previews = messages.map(|m| m.preview().bumpRef())
    return UserView(name = display_name, previews = message_previews)
}
```

## Future Development

**Proper Lifetime Detection:** The feature is intended to evolve toward proper lifetime analysis that:
- Determines optimal lifetimes for prepared references
- Avoids unnecessary memory retention
- Provides compile-time lifetime safety guarantees
- Maintains the convenience of automatic parameter detection

**Scope-Specific Allocators:** Instead of a single global bump allocator, future versions may use:
- Scope-specific allocators with appropriate lifetimes
- Automatic deallocation when references are no longer needed
- Memory pools optimized for specific usage patterns

## Benefits

1. **Clean APIs:** Functions can return reference-containing structures without exposing memory management details
2. **Automatic Management:** No manual threading of allocator parameters through call chains
3. **Performance:** Enables zero-copy patterns where references avoid data duplication
4. **Flexibility:** Functions can prepare references optimized for caller usage patterns

## Current Limitations

1. **Memory Retention:** Current implementation keeps all references alive indefinitely
2. **No Lifetime Optimization:** Cannot reclaim memory when references are no longer needed
3. **Single Allocator:** All functions share the same bump allocator instance
4. **Prototype Status:** Implementation is a placeholder for proper lifetime detection

This feature represents a stepping stone toward sophisticated lifetime management that combines the convenience of garbage collection with the performance characteristics of manual memory management.