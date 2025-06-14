use veltrano::ast::query::AstQuery;
use veltrano::{
    BinaryExpr, BinaryOp, CallExpr, Expr, FunDeclStmt, IfStmt, LiteralExpr, MethodCallExpr, Stmt,
    VarDeclStmt,
};

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

#[test]
fn test_uses_bump_allocation() {
    // Simple expression without bump
    let expr = Expr::Identifier("x".to_string());
    assert!(!AstQuery::uses_bump_allocation(&expr));

    // bumpRef() method call
    let bump_ref = Expr::MethodCall(MethodCallExpr {
        object: Box::new(Expr::Identifier("value".to_string())),
        method: "bumpRef".to_string(),
        args: vec![],
        inline_comment: None,
        id: 0,
    });
    assert!(AstQuery::uses_bump_allocation(&bump_ref));

    // Regular method call
    let regular_method = Expr::MethodCall(MethodCallExpr {
        object: Box::new(Expr::Identifier("obj".to_string())),
        method: "toString".to_string(),
        args: vec![],
        inline_comment: None,
        id: 0,
    });
    assert!(!AstQuery::uses_bump_allocation(&regular_method));

    // Binary expression with bump on left side
    let binary_with_bump = Expr::Binary(BinaryExpr {
        left: Box::new(bump_ref),
        operator: BinaryOp::Add,
        right: Box::new(Expr::Literal(LiteralExpr::Int(42))),
    });
    assert!(AstQuery::uses_bump_allocation(&binary_with_bump));
}

#[test]
fn test_stmt_uses_bump_allocation() {
    // Variable declaration with bump allocation
    let var_with_bump = Stmt::VarDecl(
        VarDeclStmt {
            name: "x".to_string(),
            type_annotation: None,
            initializer: Some(Expr::MethodCall(MethodCallExpr {
                object: Box::new(Expr::Identifier("value".to_string())),
                method: "bumpRef".to_string(),
                args: vec![],
                inline_comment: None,
                id: 0,
            })),
        },
        None,
    );
    assert!(AstQuery::stmt_uses_bump_allocation(&var_with_bump));

    // Variable declaration without bump
    let var_without_bump = Stmt::VarDecl(
        VarDeclStmt {
            name: "y".to_string(),
            type_annotation: None,
            initializer: Some(Expr::Literal(LiteralExpr::Int(42))),
        },
        None,
    );
    assert!(!AstQuery::stmt_uses_bump_allocation(&var_without_bump));

    // If statement with bump in condition
    let if_with_bump = Stmt::If(IfStmt {
        condition: Expr::MethodCall(MethodCallExpr {
            object: Box::new(Expr::Identifier("cond".to_string())),
            method: "bumpRef".to_string(),
            args: vec![],
            inline_comment: None,
            id: 0,
        }),
        then_branch: Box::new(Stmt::Block(vec![])),
        else_branch: None,
    });
    assert!(AstQuery::stmt_uses_bump_allocation(&if_with_bump));
}

#[test]
fn test_function_requires_bump() {
    // Function with bump allocation
    let fun_with_bump = FunDeclStmt {
        name: "useBump".to_string(),
        params: vec![],
        return_type: None,
        body: Box::new(Stmt::Block(vec![Stmt::Expression(
            Expr::MethodCall(MethodCallExpr {
                object: Box::new(Expr::Identifier("x".to_string())),
                method: "bumpRef".to_string(),
                args: vec![],
                inline_comment: None,
                id: 0,
            }),
            None,
        )])),
        has_hidden_bump: false,
    };
    assert!(AstQuery::function_requires_bump(&fun_with_bump));

    // Function without bump allocation
    let fun_without_bump = FunDeclStmt {
        name: "noBump".to_string(),
        params: vec![],
        return_type: None,
        body: Box::new(Stmt::Block(vec![Stmt::Return(
            Some(Expr::Literal(LiteralExpr::Int(42))),
            None,
        )])),
        has_hidden_bump: false,
    };
    assert!(!AstQuery::function_requires_bump(&fun_without_bump));
}
