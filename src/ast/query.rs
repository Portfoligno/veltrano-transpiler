// Use types re-exported in the parent module (ast/mod.rs)
use super::{Argument, Expr};
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

#[cfg(test)]
mod tests {
    use super::super::{BinaryExpr, BinaryOp, CallExpr, LiteralExpr, MethodCallExpr};
    use super::*;

    #[test]
    fn test_contains_calls() {
        // Simple identifier - no calls
        let expr = Expr::Identifier("x".to_string());
        assert!(!AstQuery::contains_calls(&expr));

        // Function call
        let call = Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier("foo".to_string())),
            args: vec![],
            is_multiline: false,
            id: 0,
        });
        assert!(AstQuery::contains_calls(&call));

        // Binary with call on left
        let binary = Expr::Binary(BinaryExpr {
            left: Box::new(call),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Literal(LiteralExpr::Int(42))),
        });
        assert!(AstQuery::contains_calls(&binary));
    }

    #[test]
    fn test_collect_identifiers() {
        // Binary expression with two identifiers
        let expr = Expr::Binary(BinaryExpr {
            left: Box::new(Expr::Identifier("x".to_string())),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Identifier("y".to_string())),
        });

        let ids = AstQuery::collect_identifiers(&expr);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("x"));
        assert!(ids.contains("y"));

        // Method call with object identifier
        let method_call = Expr::MethodCall(MethodCallExpr {
            object: Box::new(Expr::Identifier("obj".to_string())),
            method: "method".to_string(),
            args: vec![Expr::Identifier("arg".to_string())],
            inline_comment: None,
            id: 0,
        });

        let ids = AstQuery::collect_identifiers(&method_call);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("obj"));
        assert!(ids.contains("arg"));
    }

    #[test]
    fn test_count_calls() {
        // No calls
        let expr = Expr::Identifier("x".to_string());
        assert_eq!(AstQuery::count_calls(&expr), 0);

        // Single call
        let call = Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier("foo".to_string())),
            args: vec![],
            is_multiline: false,
            id: 0,
        });
        assert_eq!(AstQuery::count_calls(&call), 1);

        // Nested calls
        let nested = Expr::Call(CallExpr {
            callee: Box::new(call),
            args: vec![],
            is_multiline: false,
            id: 1,
        });
        assert_eq!(AstQuery::count_calls(&nested), 2);
    }
}
