# Dynamic Rust Interop Design

## Overview

Instead of manually registering Rust functions, we dynamically query the Rust toolchain to extract type information. This provides accurate, up-to-date type signatures for any Rust crate.

## Architecture

### 1. Dynamic Rust Registry

✅ **IMPLEMENTED** - The main orchestrator for dynamic Rust interop:

```rust
pub struct DynamicRustRegistry {
    queriers: Vec<Box<dyn RustQuerier>>,
    cache: HashMap<String, CrateInfo>,
}

impl DynamicRustRegistry {
    pub fn new() -> Self;
    
    /// Get function signature by path (e.g., "std::vec::Vec::new")
    pub fn get_function(&mut self, path: &str) -> Result<Option<FunctionInfo>, RustInteropError>;
    
    /// Get type information by path
    pub fn get_type(&mut self, path: &str) -> Result<Option<TypeInfo>, RustInteropError>;
    
    /// Add a new querier with automatic priority ordering
    pub fn add_querier(&mut self, querier: Box<dyn RustQuerier>);
}
```

### 2. Crate Information Models

```rust
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub functions: HashMap<String, FunctionInfo>,
    pub types: HashMap<String, TypeInfo>,
    pub traits: HashMap<String, TraitInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub full_path: String,
    pub generics: Vec<GenericParam>,
    pub parameters: Vec<Parameter>,
    pub return_type: RustTypeSignature,
    pub is_unsafe: bool,
    pub is_const: bool,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub full_path: String,
    pub kind: TypeKind, // Struct, Enum, Union, Trait
    pub generics: Vec<GenericParam>,
    pub methods: Vec<MethodInfo>,
    pub fields: Vec<FieldInfo>, // For structs
    pub variants: Vec<VariantInfo>, // For enums
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustTypeSignature {
    pub raw: String,              // "Option<&'a str>"
    pub parsed: Option<RustType>, // Our parsed representation
    pub lifetimes: Vec<String>,   // ["'a"]
    pub bounds: Vec<String>,      // Trait bounds like "T: Clone"
}
```

### 3. Query Implementations

#### A. rustdoc JSON Querier

✅ **IMPLEMENTED** - Phase 1 querier with highest reliability:

```rust
pub struct RustdocQuerier {
    output_dir: PathBuf,
    cache: HashMap<String, CrateInfo>,
}

impl RustdocQuerier {
    pub fn new(output_dir: Option<PathBuf>) -> Self;
    
    pub fn extract_crate_info(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        // 1. Run: cargo doc --output-format json --no-deps --target-dir <output_dir>
        // 2. Parse the generated JSON files (currently placeholder)
        // 3. Extract function/type signatures
        // 4. Convert to our internal representation
    }
}

impl RustQuerier for RustdocQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError>;
    fn supports_crate(&self, _crate_name: &str) -> bool { true }
    fn priority(&self) -> u32 { 100 } // Highest priority
}
```

#### B. rust-analyzer LSP Querier

⏳ **PENDING** - Phase 3 querier for interactive development:

```rust
pub struct RustAnalyzerQuerier {
    lsp_client: LspClient,
    workspace_root: PathBuf,
}

impl RustAnalyzerQuerier {
    pub fn query_symbol(&mut self, symbol_path: &str) -> Result<SymbolInfo, RustInteropError> {
        // 1. Send LSP "workspace/symbol" request
        // 2. Get detailed type information via "textDocument/hover"
        // 3. Parse the returned type signature
    }
    
    pub fn get_completions(&mut self, context: &CompletionContext) -> Result<Vec<CompletionItem>, RustInteropError> {
        // Get auto-completion for available methods/functions
    }
}

impl RustQuerier for RustAnalyzerQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError>;
    fn supports_crate(&self, crate_name: &str) -> bool; // Check if crate is in workspace
    fn priority(&self) -> u32 { 60 } // Lower priority than rustdoc and syn
}
```

#### C. syn Parser Querier

✅ **IMPLEMENTED** - Phase 2 querier with comprehensive source parsing:

```rust
pub struct SynQuerier {
    cargo_metadata: Option<cargo_metadata::Metadata>,
    source_cache: HashMap<PathBuf, syn::File>,
}

impl SynQuerier {
    pub fn new(manifest_path: Option<PathBuf>) -> Result<Self, RustInteropError>;
    
    pub fn extract_from_source(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError> {
        // 1. Use cargo metadata to find source files
        // 2. Parse each .rs file with syn
        // 3. Extract function/type definitions from AST
        // 4. Handle structs, enums, traits, impl blocks
    }
    
    fn parse_function(&self, item_fn: &syn::ItemFn) -> Result<FunctionInfo, RustInteropError>;
    fn parse_struct(&self, item_struct: &syn::ItemStruct) -> Result<TypeInfo, RustInteropError>;
    fn parse_enum(&self, item_enum: &syn::ItemEnum) -> Result<TypeInfo, RustInteropError>;
    fn parse_trait(&self, item_trait: &syn::ItemTrait) -> Result<TraitInfo, RustInteropError>;
    fn parse_impl_block(&self, item_impl: &syn::ItemImpl, crate_info: &mut CrateInfo) -> Result<(), RustInteropError>;
}

impl RustQuerier for SynQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError>;
    fn supports_crate(&self, _crate_name: &str) -> bool { self.cargo_metadata.is_some() }
    fn priority(&self) -> u32 { 80 } // Lower than rustdoc but higher than rust-analyzer
}
```

### 4. Unified Interface

✅ **IMPLEMENTED** - Orchestrates multiple queriers with priority ordering:

```rust
pub struct DynamicRustRegistry {
    queriers: Vec<Box<dyn RustQuerier>>,
    cache: HashMap<String, CrateInfo>,
}

pub trait RustQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, RustInteropError>;
    fn supports_crate(&self, crate_name: &str) -> bool;
    fn priority(&self) -> u32; // Higher priority queriers tried first
}

impl DynamicRustRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        
        // Add queriers in priority order (highest priority first)
        registry.add_querier(Box::new(RustdocQuerier::new(None)));
        
        // Add SynQuerier if possible (may fail if not in a Cargo project)
        if let Ok(syn_querier) = SynQuerier::new(None) {
            registry.add_querier(Box::new(syn_querier));
        }
        
        registry
    }
    
    pub fn get_function(&mut self, path: &str) -> Result<Option<FunctionInfo>, RustInteropError> {
        let (crate_name, function_path) = self.parse_path(path)?;
        
        // Try cache first
        if let Some(cached) = self.cache.get(&crate_name) {
            return Ok(cached.functions.get(&function_path).cloned());
        }
        
        // Query dynamically with priority-ordered fallback
        for querier in &mut self.queriers {
            if querier.supports_crate(&crate_name) {
                match querier.query_crate(&crate_name) {
                    Ok(crate_info) => {
                        let result = crate_info.functions.get(&function_path).cloned();
                        // Cache the result
                        self.cache.insert(crate_name, crate_info);
                        return Ok(result);
                    }
                    Err(_) => continue, // Try next querier
                }
            }
        }
        
        Ok(None)
    }
    
    pub fn get_type(&mut self, path: &str) -> Result<Option<TypeInfo>, RustInteropError> {
        // Similar implementation for type queries
    }
}
```

## Usage Examples

### In Veltrano Code

```veltrano
// Instead of manually registering, these are discovered automatically:

val numbers = Vec.new()  // Queries std::vec::Vec::new
val result = numbers.push(42)  // Queries Vec<T>::push
val s = String.from("hello")   // Queries std::string::String::from

// Even external crates work automatically:
val json = serde_json::from_str(data)  // Queries serde_json::from_str
```

### In Type Checker Integration

```rust
impl TypeInferenceEngine {
    fn infer_call_type(&mut self, call: &CallExpr) -> Result<VeltranoType, TypeCheckError> {
        if let Expr::Identifier(name) = call.callee.as_ref() {
            // Dynamic query instead of static lookup
            if let Ok(Some(func_info)) = self.rust_registry.get_function(name) {
                return Ok(self.convert_rust_signature(&func_info.return_type)?);
            }
        }
        // ... rest of inference
    }
}
```

## Benefits

1. **Automatic Discovery**: No manual registration required
2. **Always Up-to-Date**: Reflects actual Rust toolchain state  
3. **Comprehensive**: Covers entire Rust ecosystem
4. **Accurate**: Uses official Rust toolchain parsing
5. **Extensible**: Easy to add new query methods

## Implementation Status

1. ✅ **Phase 1**: rustdoc JSON querier (most reliable) - **COMPLETED**
   - Infrastructure and command execution implemented
   - Caching and error handling implemented
   - Priority: 100 (highest)
   - **TODO**: Full rustdoc JSON format parsing (currently returns placeholder data)

2. ✅ **Phase 2**: syn-based querier (most comprehensive) - **COMPLETED**
   - Full source code parsing with syn
   - Handles functions, structs, enums, traits, impl blocks
   - Cargo metadata integration for project discovery
   - Type signature conversion from syn AST to RustType
   - Priority: 80

3. ⏳ **Phase 3**: rust-analyzer querier (most interactive) - **PENDING**
   - Would provide LSP-based real-time type information
   - Priority: 60 (planned)

4. ✅ **Phase 4**: Caching and optimization - **COMPLETED**
   - File-level caching for syn parsing
   - Crate-level caching for all queriers
   - Priority-based querier ordering with fallback

## Current Capabilities

The implemented system can now:
- Automatically discover Rust type signatures from source code
- Parse complex type signatures including generics and lifetimes
- Cache results for performance
- Provide fallback between multiple querying strategies
- Extract comprehensive information about functions, types, traits, and methods

## Missing Functionality

### Trait Implementation Queries

❌ **NOT IMPLEMENTED** - Critical missing feature for type checking:

The current system cannot query which types implement which traits. This is essential for validating method calls like `.clone()` and `.toString()`.

**What we need:**
```rust
impl DynamicRustRegistry {
    /// Check if a type implements a specific trait
    pub fn type_implements_trait(
        &mut self, 
        type_path: &str,      // e.g., "i32", "std::string::String"
        trait_path: &str      // e.g., "Clone", "std::fmt::Display"
    ) -> Result<bool, RustInteropError>;
    
    /// Get all traits implemented by a type
    pub fn get_implemented_traits(
        &mut self,
        type_path: &str
    ) -> Result<Vec<String>, RustInteropError>;
}
```

**Implementation approaches:**

1. **Enhance SynQuerier** to parse `impl Trait for Type` blocks:
   ```rust
   // Currently parse_impl_block only adds methods to types
   // Need to also track: impl Clone for MyType { ... }
   ```

2. **Use rustdoc JSON** which includes trait implementations in its output

3. **Query rust-analyzer** via LSP for trait implementation information

**Impact:**
- Cannot properly type-check `.clone()` calls (must assume all types are cloneable)
- Cannot validate `.toString()` calls 
- Must skip type checking for Rust macros like `println!`

### Built-in Functions and Type Checker Integration

⚠️ **PARTIALLY IMPLEMENTED** - Built-ins exist but aren't accessible to type checker:

**Current state:**
- Built-in functions (println, print, panic, etc.) are defined in multiple places:
  - `codegen.rs`: `is_rust_macro()` for code generation
  - `rust_interop.rs`: Registry has `println` with signature
  - `type_checker.rs`: No access to built-in definitions

**Proposed solution:**
```rust
// Create a centralized built-ins module
pub mod builtins {
    pub struct BuiltinRegistry {
        functions: HashMap<String, BuiltinFunction>,
        methods: HashMap<String, BuiltinMethod>,
    }
    
    pub enum BuiltinFunction {
        // Rust macros (skip type checking)
        Println { /* variadic args */ },
        Print { /* variadic args */ },
        Panic { message: Option<VeltranoType> },
        
        // Special functions
        MutRef { value: VeltranoType }, // Generates &mut (&value).clone()
    }
    
    pub enum BuiltinMethod {
        // Universal methods (need trait checking)
        Clone,      // Requires: Clone trait
        ToString,   // Requires: ToString/Display trait
        
        // Ownership conversion methods (always available on owned types)
        Ref,        // Own<T> → T
        MutRef,     // Own<T> → MutRef<T>
        BumpRef,    // T → Str (bump allocated)
    }
}
```

**Integration approach:**
1. Type checker queries built-ins before checking user-defined functions
2. For macros: Skip type checking (as they're variadic)
3. For methods: Check trait implementations when available
4. Share definitions between type checker and codegen

This foundation makes Veltrano much more powerful by automatically understanding Rust crate APIs without manual registration.
