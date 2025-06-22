# Hard-coded Values Analysis

This document identifies hard-coded constants and string literals in the veltrano-transpiler codebase that could potentially be extracted into named constants.

**Last Updated:** June 2025

## Summary

The codebase contains several categories of hard-coded values. Most have been appropriately handled with named constants, while others are intentionally left as literals for specific reasons.

## Properly Handled Constants

The following values are defined as named constants in their respective modules:

### Indentation
- `SPACES_PER_INDENT: usize = 4` in `src/lexer.rs`
- `INDENT_STR: &str = "    "` in `src/codegen/utils.rs` and `src/codegen/expressions.rs`

### Version Defaults
- `DEFAULT_CRATE_VERSION: &str = "0.1.0"` in `src/rust_interop/syn_querier.rs`
- `DEFAULT_RUST_VERSION: &str = "1.0.0"` in `src/rust_interop/stdlib_querier.rs`

### Lifetime Annotation
- `DEFAULT_LIFETIME: &str = "'a"` in `src/codegen/types.rs`

### Comment Markers
- `DOUBLE_SLASH: &str = "//"` in codegen modules
- `SLASH_STAR: &str = "/*"` in `src/codegen/comments.rs`
- `STAR_SLASH: &str = "*/"` in `src/codegen/comments.rs`

## Intentionally Hard-coded Values

### Special Method Names
The following method names are part of the language specification and will be integrated with the built-in definitions system:
- `"ref"` → transforms to `&` operator
- `"bumpRef"` → transforms to `&'a` with lifetime
- `"mutRef"` → transforms to `&mut` operator
- `"clone"` → special clone handling
- `"toString"` → transforms to `.to_string()`

**Rationale:** These are language keywords that will be handled through the type system rather than constants.

### Type Constructor Names
- `"Unit"` type checking
- `"MutRef"` type handling

**Rationale:** Part of the language's type system design.

### Main Function Name
- `"main"` function detection

**Rationale:** Temporary implementation for bump allocation feature.

### Rust Syntax Elements
- `": "` - type annotation separator
- `", "` - parameter/argument separator  
- `"::"` - path separator
- `".vl"` - file extension

**Rationale:** Standard syntax elements that are self-documenting and unlikely to change.

## Conclusion

The codebase appropriately uses named constants for configuration values and repeated strings. The remaining hard-coded values are either:
- Part of the language specification
- Standard syntax elements
- Single-use values in clear contexts

No further extraction is recommended at this time.
