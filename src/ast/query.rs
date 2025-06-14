//! AST query functionality for common traversal operations
//!
//! Provides static methods for querying AST nodes, finding declarations, and analyzing code patterns.

// Use types re-exported in the parent module (ast/mod.rs)
use super::{Argument, Expr, FunDeclStmt, Program, Stmt, VarDeclStmt};
use std::collections::HashSet;

/// Query API for common AST traversal patterns
///
/// Provides efficient, type-safe methods for common AST analysis tasks.
/// All methods are static and operate on borrowed AST nodes.
pub struct AstQuery;

impl AstQuery {
    /// Check if an expression contains any function calls
    ///
    /// Returns true if the expression or any sub-expression is a Call or MethodCall.
    #[allow(dead_code)]
    pub fn contains_calls(expr: &Expr) -> bool {
        match expr {
            Expr::Call(_) | Expr::MethodCall(_) => true,
            Expr::Binary(b) => Self::contains_calls(&b.left) || Self::contains_calls(&b.right),
            Expr::Unary(u) => Self::contains_calls(&u.operand),
            Expr::FieldAccess(f) => Self::contains_calls(&f.object),
            _ => false,
        }
    }

    /// Get all identifiers referenced in an expression
    ///
    /// Collects all identifier names used in the expression tree.
    #[allow(dead_code)]
    pub fn collect_identifiers(expr: &Expr) -> HashSet<String> {
        let mut ids = HashSet::new();
        Self::collect_identifiers_impl(expr, &mut ids);
        ids
    }

    #[allow(dead_code)]
    fn collect_identifiers_impl(expr: &Expr, acc: &mut HashSet<String>) {
        match expr {
            Expr::Identifier(name) => {
                acc.insert(name.clone());
            }
            Expr::Binary(b) => {
                Self::collect_identifiers_impl(&b.left, acc);
                Self::collect_identifiers_impl(&b.right, acc);
            }
            Expr::Unary(u) => {
                Self::collect_identifiers_impl(&u.operand, acc);
            }
            Expr::Call(c) => {
                Self::collect_identifiers_impl(&c.callee, acc);
                for arg in &c.args {
                    match arg {
                        Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                            Self::collect_identifiers_impl(expr, acc);
                        }
                        Argument::Shorthand(field, _) => {
                            // Shorthand fields might reference identifiers
                            acc.insert(field.clone());
                        }
                        Argument::StandaloneComment(_, _) => {}
                    }
                }
            }
            Expr::MethodCall(m) => {
                Self::collect_identifiers_impl(&m.object, acc);
                for expr in &m.args {
                    Self::collect_identifiers_impl(expr, acc);
                }
            }
            Expr::FieldAccess(f) => {
                Self::collect_identifiers_impl(&f.object, acc);
            }
            Expr::Literal(_) => {}
        }
    }

    /// Count the number of function calls in an expression
    ///
    /// Counts both regular function calls and method calls.
    #[allow(dead_code)]
    pub fn count_calls(expr: &Expr) -> usize {
        match expr {
            Expr::Call(_) => {
                1 + expr_children(expr)
                    .into_iter()
                    .map(Self::count_calls)
                    .sum::<usize>()
            }
            Expr::MethodCall(_) => {
                1 + expr_children(expr)
                    .into_iter()
                    .map(Self::count_calls)
                    .sum::<usize>()
            }
            _ => expr_children(expr).into_iter().map(Self::count_calls).sum(),
        }
    }

    /// Check if an expression uses bump allocation
    ///
    /// Returns true if the expression contains any .bumpRef() method calls.
    pub fn uses_bump_allocation(expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(method_call) => {
                // Direct .bumpRef() call
                if method_call.method == "bumpRef" && method_call.args.is_empty() {
                    return true;
                }
                // Check nested expressions
                Self::uses_bump_allocation(&method_call.object)
                    || method_call.args.iter().any(Self::uses_bump_allocation)
            }
            Expr::Binary(b) => {
                Self::uses_bump_allocation(&b.left) || Self::uses_bump_allocation(&b.right)
            }
            Expr::Unary(u) => Self::uses_bump_allocation(&u.operand),
            Expr::Call(c) => {
                Self::uses_bump_allocation(&c.callee)
                    || c.args.iter().any(|arg| match arg {
                        Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                            Self::uses_bump_allocation(expr)
                        }
                        Argument::Shorthand(_, _) | Argument::StandaloneComment(_, _) => false,
                    })
            }
            Expr::FieldAccess(f) => Self::uses_bump_allocation(&f.object),
            Expr::Literal(_) | Expr::Identifier(_) => false,
        }
    }

    /// Check if a statement uses bump allocation
    ///
    /// Recursively checks all expressions within the statement tree.
    pub fn stmt_uses_bump_allocation(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expression(expr, _) => Self::uses_bump_allocation(expr),
            Stmt::VarDecl(var_decl, _) => var_decl
                .initializer
                .as_ref()
                .map_or(false, Self::uses_bump_allocation),
            Stmt::If(if_stmt) => {
                Self::uses_bump_allocation(&if_stmt.condition)
                    || Self::stmt_uses_bump_allocation(&if_stmt.then_branch)
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |s| Self::stmt_uses_bump_allocation(s))
            }
            Stmt::While(while_stmt) => {
                Self::uses_bump_allocation(&while_stmt.condition)
                    || Self::stmt_uses_bump_allocation(&while_stmt.body)
            }
            Stmt::Return(expr, _) => expr.as_ref().map_or(false, Self::uses_bump_allocation),
            Stmt::Block(statements) => statements.iter().any(Self::stmt_uses_bump_allocation),
            Stmt::FunDecl(_) | Stmt::Comment(_) | Stmt::Import(_) | Stmt::DataClass(_) => false,
        }
    }

    /// Check if a function requires bump allocation
    ///
    /// Checks if the function body contains any bump allocation usage.
    pub fn function_requires_bump(fun_decl: &FunDeclStmt) -> bool {
        Self::stmt_uses_bump_allocation(&fun_decl.body)
    }

    /// Find all variable declarations in a statement
    ///
    /// Recursively collects all VarDeclStmt nodes within the statement tree.
    #[allow(dead_code)]
    pub fn find_var_decls(stmt: &Stmt) -> Vec<&VarDeclStmt> {
        let mut decls = Vec::new();
        Self::collect_var_decls(stmt, &mut decls);
        decls
    }

    #[allow(dead_code)]
    fn collect_var_decls<'a>(stmt: &'a Stmt, acc: &mut Vec<&'a VarDeclStmt>) {
        match stmt {
            Stmt::VarDecl(var_decl, _) => acc.push(var_decl),
            Stmt::Block(statements) => {
                for s in statements {
                    Self::collect_var_decls(s, acc);
                }
            }
            Stmt::If(if_stmt) => {
                Self::collect_var_decls(&if_stmt.then_branch, acc);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_var_decls(else_branch, acc);
                }
            }
            Stmt::While(while_stmt) => {
                Self::collect_var_decls(&while_stmt.body, acc);
            }
            _ => {}
        }
    }

    /// Find all function declarations in a statement
    pub fn find_function_decls(stmt: &Stmt) -> Vec<&FunDeclStmt> {
        let mut decls = Vec::new();
        Self::collect_function_decls(stmt, &mut decls);
        decls
    }

    fn collect_function_decls<'a>(stmt: &'a Stmt, acc: &mut Vec<&'a FunDeclStmt>) {
        match stmt {
            Stmt::FunDecl(fun_decl) => acc.push(fun_decl),
            Stmt::Block(statements) => {
                for s in statements {
                    Self::collect_function_decls(s, acc);
                }
            }
            Stmt::If(if_stmt) => {
                Self::collect_function_decls(&if_stmt.then_branch, acc);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_function_decls(else_branch, acc);
                }
            }
            Stmt::While(while_stmt) => {
                Self::collect_function_decls(&while_stmt.body, acc);
            }
            _ => {}
        }
    }

    /// Collect all variable references (identifiers) in a statement
    #[allow(dead_code)]
    pub fn collect_variable_references(stmt: &Stmt) -> HashSet<String> {
        let mut refs = HashSet::new();
        Self::collect_stmt_variable_refs(stmt, &mut refs);
        refs
    }

    #[allow(dead_code)]
    fn collect_stmt_variable_refs(stmt: &Stmt, acc: &mut HashSet<String>) {
        match stmt {
            Stmt::Expression(expr, _) => {
                acc.extend(Self::collect_identifiers(expr));
            }
            Stmt::VarDecl(var_decl, _) => {
                if let Some(init) = &var_decl.initializer {
                    acc.extend(Self::collect_identifiers(init));
                }
            }
            Stmt::Return(Some(expr), _) => {
                acc.extend(Self::collect_identifiers(expr));
            }
            Stmt::If(if_stmt) => {
                acc.extend(Self::collect_identifiers(&if_stmt.condition));
                Self::collect_stmt_variable_refs(&if_stmt.then_branch, acc);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_stmt_variable_refs(else_branch, acc);
                }
            }
            Stmt::While(while_stmt) => {
                acc.extend(Self::collect_identifiers(&while_stmt.condition));
                Self::collect_stmt_variable_refs(&while_stmt.body, acc);
            }
            Stmt::Block(statements) => {
                for s in statements {
                    Self::collect_stmt_variable_refs(s, acc);
                }
            }
            Stmt::FunDecl(fun_decl) => {
                Self::collect_stmt_variable_refs(&fun_decl.body, acc);
            }
            _ => {}
        }
    }

    /// Find function declarations at the top level of a program
    #[allow(dead_code)]
    pub fn find_program_functions(program: &Program) -> Vec<&FunDeclStmt> {
        program
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::FunDecl(fun_decl) => Some(fun_decl),
                _ => None,
            })
            .collect()
    }

    /// Find all variable declarations in a program
    #[allow(dead_code)]
    pub fn find_program_variables(program: &Program) -> Vec<&VarDeclStmt> {
        let mut vars = Vec::new();
        for stmt in &program.statements {
            vars.extend(Self::find_var_decls(stmt));
        }
        vars
    }
}

// Helper function to get child expressions
#[allow(dead_code)]
fn expr_children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::Binary(b) => vec![&b.left, &b.right],
        Expr::Unary(u) => vec![&u.operand],
        Expr::Call(c) => {
            let mut children = vec![&*c.callee];
            for arg in &c.args {
                match arg {
                    Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                        children.push(expr);
                    }
                    _ => {}
                }
            }
            children
        }
        Expr::MethodCall(m) => {
            let mut children = vec![&*m.object];
            children.extend(m.args.iter());
            children
        }
        Expr::FieldAccess(f) => vec![&f.object],
        Expr::Literal(_) | Expr::Identifier(_) => vec![],
    }
}
