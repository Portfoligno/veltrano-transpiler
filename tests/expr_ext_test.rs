use veltrano::error::{SourceLocation, Span};
use veltrano::{
    BinaryExpr, BinaryOp, CallExpr, Expr, ExprExt, LiteralExpr, Located, LocatedExpr,
    MethodCallExpr,
};

// Helper function to create a test located expression
fn loc(expr: Expr) -> LocatedExpr {
    Located::new(expr, Span::single(SourceLocation::new(1, 1)))
}

#[test]
fn test_walk_pre_order() {
    // Create a binary expression: 1 + 2
    let expr = loc(Expr::Binary(BinaryExpr {
        left: Box::new(loc(Expr::Literal(LiteralExpr::Int(1)))),
        comment_after_left: None,
        operator: BinaryOp::Add,
        comment_after_operator: None,
        right: Box::new(loc(Expr::Literal(LiteralExpr::Int(2)))),
    }));

    let mut visited = Vec::new();
    let result = expr.walk(&mut |e| {
        match &e.node {
            Expr::Binary(_) => visited.push("binary"),
            Expr::Literal(LiteralExpr::Int(n)) => visited.push(match n {
                1 => "1",
                2 => "2",
                _ => "other",
            }),
            _ => visited.push("other"),
        }
        Ok::<(), ()>(())
    });

    assert!(result.is_ok());
    assert_eq!(visited, vec!["binary", "1", "2"]);
}

#[test]
fn test_walk_post_order() {
    // Create a binary expression: 1 + 2
    let expr = loc(Expr::Binary(BinaryExpr {
        left: Box::new(loc(Expr::Literal(LiteralExpr::Int(1)))),
        comment_after_left: None,
        operator: BinaryOp::Add,
        comment_after_operator: None,
        right: Box::new(loc(Expr::Literal(LiteralExpr::Int(2)))),
    }));

    let mut visited = Vec::new();
    let result = expr.walk_post(&mut |e| {
        match &e.node {
            Expr::Binary(_) => visited.push("binary"),
            Expr::Literal(LiteralExpr::Int(n)) => visited.push(match n {
                1 => "1",
                2 => "2",
                _ => "other",
            }),
            _ => visited.push("other"),
        }
        Ok::<(), ()>(())
    });

    assert!(result.is_ok());
    assert_eq!(visited, vec!["1", "2", "binary"]);
}

#[test]
fn test_walk_early_exit() {
    // Create nested expression: foo(1 + 2)
    let expr = loc(Expr::Call(CallExpr {
        callee: Box::new(loc(Expr::Identifier("foo".to_string()))),
        args: vec![veltrano::Argument::Bare(
            loc(Expr::Binary(BinaryExpr {
                left: Box::new(loc(Expr::Literal(LiteralExpr::Int(1)))),
                comment_after_left: None,
                operator: BinaryOp::Add,
                comment_after_operator: None,
                right: Box::new(loc(Expr::Literal(LiteralExpr::Int(2)))),
            })),
            veltrano::ArgumentComment {
                before: None,
                after: None,
            },
        )],
        is_multiline: false,
        id: 0,
    }));

    let mut count = 0;
    let result: Result<(), &str> = expr.walk(&mut |e| {
        count += 1;
        if matches!(&e.node, Expr::Binary(_)) {
            Err("found binary")
        } else {
            Ok(())
        }
    });

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "found binary");
    assert_eq!(count, 3); // Call, Identifier, Binary
}

#[test]
fn test_find_subexpressions() {
    // Create expression: obj.method(x, y)
    let expr = loc(Expr::MethodCall(MethodCallExpr {
        object: Box::new(loc(Expr::Identifier("obj".to_string()))),
        method: "method".to_string(),
        args: vec![
            loc(Expr::Identifier("x".to_string())),
            loc(Expr::Identifier("y".to_string())),
        ],
        inline_comment: None,
        id: 0,
    }));

    // Find all identifiers
    let identifiers = expr.find_subexpressions(|e| matches!(&e.node, Expr::Identifier(_)));
    assert_eq!(identifiers.len(), 3);

    // Extract names
    let names: Vec<&str> = identifiers
        .iter()
        .filter_map(|e| match &e.node {
            Expr::Identifier(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["obj", "x", "y"]);
}

#[test]
fn test_any_subexpr() {
    // Create expression with nested literals
    let expr = loc(Expr::Binary(BinaryExpr {
        left: Box::new(loc(Expr::Literal(LiteralExpr::Int(42)))),
        comment_after_left: None,
        operator: BinaryOp::Add,
        comment_after_operator: None,
        right: Box::new(loc(Expr::Identifier("x".to_string()))),
    }));

    // Check if any sub-expression is an integer literal
    assert!(expr.any_subexpr(|e| matches!(&e.node, Expr::Literal(LiteralExpr::Int(_)))));

    // Check if any sub-expression is a string literal
    assert!(!expr.any_subexpr(|e| matches!(&e.node, Expr::Literal(LiteralExpr::String(_)))));
}

#[test]
fn test_all_subexprs() {
    // Create expression with only identifiers
    let expr = loc(Expr::Binary(BinaryExpr {
        left: Box::new(loc(Expr::Identifier("a".to_string()))),
        comment_after_left: None,
        operator: BinaryOp::Add,
        comment_after_operator: None,
        right: Box::new(loc(Expr::Identifier("b".to_string()))),
    }));

    // Check if all sub-expressions are identifiers or binary
    assert!(expr.all_subexprs(|e| { matches!(&e.node, Expr::Identifier(_) | Expr::Binary(_)) }));

    // Check if all sub-expressions are identifiers (should be false because of Binary)
    assert!(!expr.all_subexprs(|e| matches!(&e.node, Expr::Identifier(_))));
}

#[test]
fn test_complex_traversal() {
    // Create complex expression: foo(a + b, c.method(d))
    let expr = loc(Expr::Call(CallExpr {
        callee: Box::new(loc(Expr::Identifier("foo".to_string()))),
        args: vec![
            veltrano::Argument::Bare(
                loc(Expr::Binary(BinaryExpr {
                    left: Box::new(loc(Expr::Identifier("a".to_string()))),
                    comment_after_left: None,
                    operator: BinaryOp::Add,
                    comment_after_operator: None,
                    right: Box::new(loc(Expr::Identifier("b".to_string()))),
                })),
                veltrano::ArgumentComment {
                    before: None,
                    after: None,
                },
            ),
            veltrano::Argument::Bare(
                loc(Expr::MethodCall(MethodCallExpr {
                    object: Box::new(loc(Expr::Identifier("c".to_string()))),
                    method: "method".to_string(),
                    args: vec![loc(Expr::Identifier("d".to_string()))],
                    inline_comment: None,
                    id: 0,
                })),
                veltrano::ArgumentComment {
                    before: None,
                    after: None,
                },
            ),
        ],
        is_multiline: false,
        id: 0,
    }));

    // Count different types of expressions
    let mut call_count = 0;
    let mut method_count = 0;
    let mut binary_count = 0;
    let mut identifier_count = 0;

    let _ = expr.walk(&mut |e| {
        match &e.node {
            Expr::Call(_) => call_count += 1,
            Expr::MethodCall(_) => method_count += 1,
            Expr::Binary(_) => binary_count += 1,
            Expr::Identifier(_) => identifier_count += 1,
            _ => {}
        }
        Ok::<(), ()>(())
    });

    assert_eq!(call_count, 1);
    assert_eq!(method_count, 1);
    assert_eq!(binary_count, 1);
    assert_eq!(identifier_count, 5); // foo, a, b, c, d
}
