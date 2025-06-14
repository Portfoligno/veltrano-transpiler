use veltrano::ast::query::AstQuery;
use veltrano::{
    BinaryExpr, BinaryOp, CallExpr, Expr, FunDeclStmt, IfStmt, LiteralExpr, MethodCallExpr,
    Program, Stmt, VarDeclStmt,
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

#[test]
fn test_find_var_decls() {
    // Create a block with multiple variable declarations
    let block = Stmt::Block(vec![
        Stmt::VarDecl(
            VarDeclStmt {
                name: "x".to_string(),
                type_annotation: None,
                initializer: Some(Expr::Literal(LiteralExpr::Int(42))),
            },
            None,
        ),
        Stmt::VarDecl(
            VarDeclStmt {
                name: "y".to_string(),
                type_annotation: None,
                initializer: Some(Expr::Identifier("x".to_string())),
            },
            None,
        ),
        Stmt::If(IfStmt {
            condition: Expr::Identifier("condition".to_string()),
            then_branch: Box::new(Stmt::VarDecl(
                VarDeclStmt {
                    name: "z".to_string(),
                    type_annotation: None,
                    initializer: Some(Expr::Literal(LiteralExpr::Bool(true))),
                },
                None,
            )),
            else_branch: None,
        }),
    ]);

    let vars = AstQuery::find_var_decls(&block);
    assert_eq!(vars.len(), 3);
    assert_eq!(vars[0].name, "x");
    assert_eq!(vars[1].name, "y");
    assert_eq!(vars[2].name, "z");
}

#[test]
fn test_find_function_decls() {
    // Create a block with nested function declarations
    let block = Stmt::Block(vec![
        Stmt::FunDecl(FunDeclStmt {
            name: "foo".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(Stmt::Block(vec![])),
            has_hidden_bump: false,
        }),
        Stmt::If(IfStmt {
            condition: Expr::Literal(LiteralExpr::Bool(true)),
            then_branch: Box::new(Stmt::FunDecl(FunDeclStmt {
                name: "bar".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(Stmt::Block(vec![])),
                has_hidden_bump: false,
            })),
            else_branch: None,
        }),
    ]);

    let funs = AstQuery::find_function_decls(&block);
    assert_eq!(funs.len(), 2);
    assert_eq!(funs[0].name, "foo");
    assert_eq!(funs[1].name, "bar");
}

#[test]
fn test_collect_variable_references() {
    // Create a statement with various variable references
    let stmt = Stmt::Block(vec![
        Stmt::VarDecl(
            VarDeclStmt {
                name: "x".to_string(),
                type_annotation: None,
                initializer: Some(Expr::Binary(BinaryExpr {
                    left: Box::new(Expr::Identifier("a".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Identifier("b".to_string())),
                })),
            },
            None,
        ),
        Stmt::If(IfStmt {
            condition: Expr::Identifier("c".to_string()),
            then_branch: Box::new(Stmt::Expression(
                Expr::MethodCall(MethodCallExpr {
                    object: Box::new(Expr::Identifier("d".to_string())),
                    method: "method".to_string(),
                    args: vec![Expr::Identifier("e".to_string())],
                    inline_comment: None,
                    id: 0,
                }),
                None,
            )),
            else_branch: None,
        }),
    ]);

    let refs = AstQuery::collect_variable_references(&stmt);
    assert_eq!(refs.len(), 5);
    assert!(refs.contains("a"));
    assert!(refs.contains("b"));
    assert!(refs.contains("c"));
    assert!(refs.contains("d"));
    assert!(refs.contains("e"));
}

#[test]
fn test_find_program_functions() {
    let program = Program {
        statements: vec![
            Stmt::VarDecl(
                VarDeclStmt {
                    name: "global".to_string(),
                    type_annotation: None,
                    initializer: None,
                },
                None,
            ),
            Stmt::FunDecl(FunDeclStmt {
                name: "main".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(Stmt::Block(vec![])),
                has_hidden_bump: false,
            }),
            Stmt::FunDecl(FunDeclStmt {
                name: "helper".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(Stmt::Block(vec![])),
                has_hidden_bump: false,
            }),
        ],
    };

    let funs = AstQuery::find_program_functions(&program);
    assert_eq!(funs.len(), 2);
    assert_eq!(funs[0].name, "main");
    assert_eq!(funs[1].name, "helper");
}
