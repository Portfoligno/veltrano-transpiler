# Hard-coded Values Analysis

This document identifies hard-coded numeric constants and string literals in the veltrano-transpiler codebase that could be extracted into named constants for better maintainability.

## Summary

The codebase contains several categories of hard-coded values:
- Numeric constants for formatting (indentation)
- String literals for special method names and transformations
- Hard-coded version strings
- Rust syntax elements
- File extensions and type names

## Numeric Constants

### 1. Indentation Constants
**Current Usage:**
- `src/lexer.rs:381,383` - Uses `4` for tracking indentation levels
- `src/codegen.rs:492,527,674` - Uses `4` for generating indented output

**Recommendation:**
```rust
// In a constants module or at the top of relevant files
pub const SPACES_PER_INDENT: usize = 4;
pub const INDENT_STR: &str = "    ";  // Pre-computed 4-space string
```

### 2. Default Version Numbers
**Current Usage:**
- `src/rust_interop/compiler.rs:466` - `"0.1.0"` as default crate version
- `src/rust_interop/mod.rs:231` - `"1.0.0"` as default version

**Recommendation:**
```rust
pub const DEFAULT_CRATE_VERSION: &str = "0.1.0";
pub const DEFAULT_RUST_VERSION: &str = "1.0.0";
```

## String Literals

### ~~1. Special Method Names~~
These method names trigger special transpilation behavior and are critical to the language design.

**Current Usage in `src/codegen.rs:1159-1181`:**
- `"ref"` → transforms to `&` operator
- `"bumpRef"` → transforms to `&'a` with lifetime
- `"mutRef"` → transforms to `&mut` operator
- `"clone"` → special clone handling
- `"toString"` → transforms to `.to_string()`

**Note:** These special method names are planned to be integrated further with the built-in definitions system. They should be **left out from extraction as constants** as they will be handled through a different mechanism in the future.

**Current Recommendation:** Leave as-is until the built-in integration plan is implemented.

### ~~2. Main Function Name~~
**Current Usage:**
- `src/ast_types.rs:166,197` - Checking for "main" function
- `src/codegen.rs:323` - Special handling for main function

**Note:** These hardcoded checks for "main" are temporary workarounds for the work-in-progress bump allocation feature. They are expected to be removed once the feature is properly implemented, so extracting them as constants would be premature.

### ~~3. Type Constructor Names~~
**Current Usage:**
- `src/parser.rs:1027` - `"Unit"` type checking
- `src/codegen.rs:999` - `"MutRef"` type handling

**Note:** Similar to special method names, type constructor names are part of the language's type system design and will be integrated with the built-in type definitions. They should be **left out from extraction as constants**.

**Current Recommendation:** Leave as-is until the type system integration plan is implemented.

### 4. Lifetime Annotations
**Current Usage:**
- `src/codegen.rs:613,668` - `"'a"` as default lifetime

**Recommendation:**
```rust
pub const DEFAULT_LIFETIME: &str = "'a";
```

### 5. Comment Markers
**Current Usage:**
- `src/lexer.rs` uses character-by-character parsing (clear in context, no extraction needed)
- `src/codegen.rs` has 8 occurrences of comment markers as strings for output generation:
  - `"//"` - 6 occurrences (lines 487, 522, 833, 949, 1223, 1245)
  - `"/*"` - 1 occurrence (line 1219)
  - `"*/"` - 1 occurrence (line 1221)

**Note:** The fact that Veltrano and Rust use the same comment syntax is coincidental. The lexer detects and strips comment markers during parsing, and codegen re-adds them during output.

**Recommendation:** Since codegen has multiple occurrences, consider local constants within the codegen module only:
```rust
// Within codegen.rs (for string output)
const DOUBLE_SLASH: &str = "//";
const SLASH_STAR: &str = "/*";
const STAR_SLASH: &str = "*/";
```

### ~~6. File Extensions~~
**Current Usage:**
- `src/main.rs:193` - `".vl"` file extension check

**Note:** Single occurrence in a clear context. No extraction needed.

### 7. Rust Syntax Elements
**Current Usage in `src/codegen.rs`:**
- `": "` - type annotation separator (6 occurrences: lines 252, 412, 700, 723, 763, 957)
  - Used for parameter types, struct field types, variable type annotations
- `", "` - parameter/argument separator (6 occurrences: lines 695, 748, 850, 1047, 1138, 1155)
  - Used in function parameters, tuple elements, array elements
- `"::"` - path separator (4 occurrences: lines 911, 1061, 1129, 1146)
  - Used for module paths like `std::vec::Vec`

**Note:** These are standard Rust syntax elements that are universally recognized and unlikely to change.

**Recommendation:** Low priority - only extract if absolute consistency is desired:
```rust
// Within codegen.rs (if needed for consistency)
const COLON_SPACE: &str = ": ";
const COMMA_SPACE: &str = ", ";
const DOUBLE_COLON: &str = "::";
```

## Implementation Priority

### High Priority (Core Language Semantics)
1. **Indentation constants** - Used throughout formatting

Note: Excluded from extraction:
- Special method names & type constructor names - will be integrated with built-in definitions
- Main function name - temporary workaround for WIP bump allocation feature

### Medium Priority (Clarity and Maintainability)
1. **Default versions** - Used in multiple places
2. **Default lifetime** - Used in code generation

### Low Priority (Nice to Have)
1. **Rust syntax strings** - Standard and unlikely to change
2. **Comment markers** - Local to codegen module, well-established syntax

## Benefits of Extraction

1. **Centralized Configuration**: All language-specific constants in one place
2. **Easier Refactoring**: Change values in one location
3. **Self-Documenting**: Named constants explain their purpose
4. **Prevent Typos**: Using constants prevents string literal typos
5. **Consistency**: Ensures the same value is used everywhere

## Suggested Implementation Approach

Instead of a central constants module, define constants locally in the modules where they're used:

```rust
// Within src/lexer.rs
const SPACES_PER_INDENT: usize = 4;

// Within src/codegen.rs
const SPACES_PER_INDENT: usize = 4;
const INDENT_STR: &str = "    ";
const DEFAULT_LIFETIME: &str = "'a";
const DOUBLE_SLASH: &str = "//";
const SLASH_STAR: &str = "/*";
const STAR_SLASH: &str = "*/";

// Within src/rust_interop/compiler.rs
const DEFAULT_CRATE_VERSION: &str = "0.1.0";

// Within src/rust_interop/mod.rs
const DEFAULT_RUST_VERSION: &str = "1.0.0";
```

This approach:
- Keeps constants close to where they're used
- Avoids creating dependencies on a central module
- Makes each module more self-contained
- Allows different modules to have different values if needed

## Conclusion

While the codebase is generally well-structured, extracting these hard-coded values would improve maintainability and make the code more self-documenting. The highest priority should be given to values used in multiple locations (indentation constants) and configuration values (default versions). Using local constants rather than a central module keeps the code modular and avoids unnecessary dependencies.
