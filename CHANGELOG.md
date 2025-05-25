# Changelog

All notable changes to the Veltrano Transpiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

