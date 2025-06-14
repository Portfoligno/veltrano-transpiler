// Use types re-exported in the parent module (ast/mod.rs)
use super::{Argument, Expr, FunDeclStmt, Stmt};
use std::collections::HashSet;

/// Query API for common AST traversal patterns
pub struct AstQuery;

impl AstQuery {
    /// Check if an expression contains any function calls
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
    pub fn collect_identifiers(expr: &Expr) -> HashSet<String> {
        let mut ids = HashSet::new();
        Self::collect_identifiers_impl(expr, &mut ids);
        ids
    }

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

    /// Check if an expression uses bump allocation (calls .bumpRef())
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

    /// Check if a function requires bump allocation based on its body
    pub fn function_requires_bump(fun_decl: &FunDeclStmt) -> bool {
        Self::stmt_uses_bump_allocation(&fun_decl.body)
    }
}

// Helper function to get child expressions
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
