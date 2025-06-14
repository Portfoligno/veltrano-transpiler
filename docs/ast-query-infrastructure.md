# AST Query Infrastructure Documentation

## Overview

The AST Query Infrastructure provides a type-safe, efficient way to traverse and query the Veltrano Abstract Syntax Tree (AST). It consists of two main components:

1. **AstQuery API** - Static methods for common AST queries
2. **Extension Traits** - Traversal methods added to Expr and Stmt types

## Design Philosophy

The query infrastructure was designed with these principles:

- **Type Safety**: Leverage Rust's type system to prevent errors
- **Composability**: Small, focused methods that can be combined
- **Performance**: Early exit capabilities and efficient traversal
- **Ergonomics**: Natural API that fits Rust idioms

## AstQuery API

The `AstQuery` struct provides static methods for common AST queries:

### Expression Queries

```rust
// Check if an expression contains function calls
AstQuery::contains_calls(&expr) -> bool

// Collect all identifiers in an expression
AstQuery::collect_identifiers(&expr) -> HashSet<String>

// Count function calls in an expression
AstQuery::count_calls(&expr) -> usize

// Check if expression uses bump allocation
AstQuery::uses_bump_allocation(&expr) -> bool
```

### Statement Queries

```rust
// Check if statement uses bump allocation
AstQuery::stmt_uses_bump_allocation(&stmt) -> bool

// Check if function requires bump parameter
AstQuery::function_requires_bump(&fun_decl) -> bool

// Find all variable declarations
AstQuery::find_var_decls(&stmt) -> Vec<&VarDeclStmt>

// Find all function declarations
AstQuery::find_function_decls(&stmt) -> Vec<&FunDeclStmt>

// Collect all variable references
AstQuery::collect_variable_references(&stmt) -> HashSet<String>
```

### Program Queries

```rust
// Find all top-level functions
AstQuery::find_program_functions(&program) -> Vec<&FunDeclStmt>

// Find all top-level variables
AstQuery::find_program_variables(&program) -> Vec<&VarDeclStmt>
```

## Extension Traits

### ExprExt Trait

Adds traversal methods to all `Expr` types:

```rust
// Pre-order traversal with early exit
expr.walk(&mut |e| {
    // Return Err to stop traversal
    Ok(())
})

// Post-order traversal
expr.walk_post(&mut |e| {
    // Process after children
    Ok(())
})

// Find matching sub-expressions
let calls = expr.find_subexpressions(|e| matches!(e, Expr::Call(_)));

// Check predicates
if expr.any_subexpr(|e| matches!(e, Expr::MethodCall(_))) { }
if expr.all_subexprs(|e| !matches!(e, Expr::Literal(_))) { }
```

### StmtExt Trait

Adds traversal methods to all `Stmt` types:

```rust
// Pre-order statement traversal
stmt.walk(&mut |s| {
    // Visit each statement
    Ok(())
})

// Post-order statement traversal
stmt.walk_post(&mut |s| {
    // Process after children
    Ok(())
})

// Find matching statements
let functions = stmt.find_statements(|s| matches!(s, Stmt::FunDecl(_)));

// Walk all expressions within statements
stmt.walk_expressions(&mut |expr| {
    // Process each expression
    Ok(())
})

// Check for early exit
if stmt.can_exit_early() {
    // Statement contains return
}
```

## Usage Examples

### Example 1: Collecting Function Information

```rust
use crate::ast::query::AstQuery;

// Find all functions that use bump allocation
let functions = AstQuery::find_program_functions(&program);
let bump_functions: Vec<_> = functions
    .into_iter()
    .filter(|f| AstQuery::function_requires_bump(f))
    .collect();
```

### Example 2: Expression Analysis

```rust
use crate::ast_types::ExprExt;

// Count method calls in an expression
let mut method_count = 0;
expr.walk(&mut |e| {
    if matches!(e, Expr::MethodCall(_)) {
        method_count += 1;
    }
    Ok::<(), ()>(())
});
```

### Example 3: Type Checker Integration

```rust
// Find all function declarations for signature collection
let function_decls = AstQuery::find_function_decls(stmt);
for fun_decl in function_decls {
    self.collect_function_signature(fun_decl)?;
}
```

### Example 4: Code Generation

```rust
// Check if function needs bump parameter
if AstQuery::function_requires_bump(fun_decl) {
    params.push("bump: &Bump");
}

// Walk expressions to find bump usage
let mut uses_bump = false;
fun_decl.body.walk_expressions(&mut |expr| {
    if let Expr::Call(call) = expr {
        if let Expr::Identifier(name) = call.callee.as_ref() {
            if functions_with_bump.contains(name) {
                uses_bump = true;
                return Err(()); // Early exit
            }
        }
    }
    Ok::<(), ()>(())
});
```

## Migration Guide

### Before (Manual Traversal)

```rust
fn collect_functions(stmt: &Stmt) -> Vec<&FunDeclStmt> {
    match stmt {
        Stmt::FunDecl(f) => vec![f],
        Stmt::Block(stmts) => {
            stmts.iter().flat_map(collect_functions).collect()
        }
        Stmt::If(if_stmt) => {
            let mut funcs = collect_functions(&if_stmt.then_branch);
            if let Some(else_branch) = &if_stmt.else_branch {
                funcs.extend(collect_functions(else_branch));
            }
            funcs
        }
        // ... more cases
        _ => vec![]
    }
}
```

### After (Query Infrastructure)

```rust
let functions = AstQuery::find_function_decls(stmt);
```

## Performance Considerations

1. **Early Exit**: Use `Err` in traversal callbacks to stop early
2. **Lazy Evaluation**: Methods like `any_subexpr` stop at first match
3. **Direct Access**: Query methods avoid intermediate allocations where possible

## Future Extensions

The query infrastructure is designed to be extensible:

- Additional query methods can be added to `AstQuery`
- New traversal methods can be added to extension traits
- Specialized iterators could be implemented for common patterns

## Testing

All query methods have comprehensive tests in:
- `/tests/ast_query_test.rs` - AstQuery methods
- `/tests/expr_ext_test.rs` - ExprExt trait
- `/tests/stmt_ext_test.rs` - StmtExt trait
