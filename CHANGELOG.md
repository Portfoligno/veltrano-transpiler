# Changelog

All notable changes to the Veltrano Transpiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-05-29

### Added
- Import statement support for UFCS transformation
  - Syntax: `import Type.method [as alias]`
  - Method calls transform to UFCS when imported: `obj.method()` → `Type::method(obj)`
  - Function calls also transform when imported: `func()` → `Type::func()`
  - Supports aliasing: `import Vec.new as newVec` allows `newVec()` → `Vec::new()`
  - Explicit imports override pre-imported methods (e.g., `import MyClone.clone`)
  - Local functions have priority over imports
  - Import statements don't generate Rust code, they control transpilation behavior
- `.toString()` now uses UFCS: generates `ToString::to_string(obj)` for consistency with `.clone()`

## [0.2.0] - 2025-01-28

### Added
- Unit literal support - `Unit` can now be used as a literal expression, not just a type
- Unary expression support with negation (`-`) operator
  - Double minus (`--`) requires parentheses: `-(-x)` instead of `--x`
- `MutRef<T>` type that transpiles to `&mut T` or `&mut &T` depending on context
- `MutRef(value)` function that generates `&mut (&value).clone()` - always clones the value
- `.mutRef()` method that generates `&mut value` - requires explicit `.clone()` if needed
- `Own<T>` type constructor for explicit ownership control
- UFCS (Uniform Function Call Syntax) for `.clone()` - generates `Clone::clone()` to avoid auto-ref issues

### Changed
- **BREAKING**: Complete type system redesign from enum-based to struct with reference depth tracking
  - Types now composed of `BaseType` + `reference_depth` for better composability
  - `Ref<T>` adds to reference depth, `Own<T>` subtracts from it (symmetric operations)
- **BREAKING**: Reference-by-default semantics - String/Str/Custom types now transpile to references
  - `String` → `&String`, `Str` → `&str`, `Custom` → `&Custom`
  - Use `Own<String>` to get owned `String` type

## [0.1.8] - 2025-01-26

### Added
- `--preserve-comments` CLI flag to optionally include comments in generated Rust code
- Comments can now be preserved in the output when explicitly requested

### Changed
- Updated CLI to support options before the input file (e.g., `veltrano --preserve-comments input.vl`)
- Enhanced help text to show available options with examples

## [0.1.7] - 2025-01-26

### Removed
- Removed `var` keyword support - only `val` (immutable variables) are now supported
- This change reflects the semantic mismatch between Kotlin's `var` (rebinding only) and Rust's `let mut` (recursive mutability)

### Added
- Design notes in README explaining the rationale for removing `var` keyword
- Clarification about `Ref<T>` naming choice and its relationship to Rust's idioms

## [0.1.6] - 2025-01-25

### Changed
- Improved parser error messages with detailed location and token information
- Refactored lexer to eliminate code duplication using generic `read_while` method
- Refactored parser to remove duplicate binary expression parsing logic
- Refactored code generator to consolidate inline comment generation
- Enhanced test framework to properly fail on parse errors instead of warnings
- Removed 'v' prefix from version display in help message

### Internal
- Reduced codebase by ~165 lines through systematic refactoring
- Applied consistent code formatting across all Rust files

## [0.1.5] - 2025-01-25

### Added
- Initial release of Veltrano transpiler
- Core language features:
  - Variable declarations (mutable and immutable)
  - Function declarations with parameters and return types
  - Basic types: Int, String, Bool, Float, Never (!)
  - Control flow: if/else statements, while loops
  - String literals and numeric literals
  - Boolean expressions and operators
  - Method calls on types (e.g., `.toString()`)
- Comment preservation:
  - Line comments (`//`) preserved with original formatting
  - Block comments (`/* */`) preserved with original formatting
  - Inline comments after statements maintained
- Transpilation features:
  - Converts `while(true)` to Rust's `loop` construct
  - Proper type inference and annotation translation
  - Maintains code structure and formatting intent
- CLI features:
  - `--version`/`-v` flag to display version
  - `--help`/`-h` flag to show usage information
  - Simple file-based transpilation interface
- Development tooling:
  - Comprehensive test suite
  - Example programs demonstrating language features
  - Cargo-based build system
- Documentation:
  - README with language overview and examples
  - CLAUDE.md with development guidelines
  - Example programs in examples/ directory

### Known Issues
- Some test cases may need adjustment for comment preservation


