use veltrano::error::{SourceLocation, Span};
use veltrano::{
    BinaryExpr, BinaryOp, Expr, FunDeclStmt, IfStmt, LiteralExpr, Located, LocatedExpr, Stmt,
    StmtExt, VarDeclStmt, WhileStmt,
};

// Helper function to create a test located expression
fn loc(expr: Expr) -> LocatedExpr {
    Located::new(expr, Span::single(SourceLocation::new(1, 1)))
}

#[test]
fn test_walk_statements() {
    // Create a block with various statement types
    let block = Stmt::Block(vec![
        Stmt::VarDecl(VarDeclStmt {
            name: "x".to_string(),
            type_annotation: None,
            initializer: Some(loc(Expr::Literal(LiteralExpr::Int(42)))),
        }),
        Stmt::Expression(loc(Expr::Identifier("x".to_string()))),
        Stmt::Return(Some(loc(Expr::Identifier("x".to_string())))),
    ]);

    let mut visited = Vec::new();
    let result = block.walk(&mut |stmt| {
        match stmt {
            Stmt::Block(_) => visited.push("block"),
            Stmt::VarDecl(_) => visited.push("var_decl"),
            Stmt::Expression(_) => visited.push("expression"),
            Stmt::Return(_) => visited.push("return"),
            _ => visited.push("other"),
        }
        Ok::<(), ()>(())
    });

    assert!(result.is_ok());
    assert_eq!(visited, vec!["block", "var_decl", "expression", "return"]);
}

#[test]
fn test_walk_post_order_statements() {
    // Create an if statement with blocks
    let if_stmt = Stmt::If(IfStmt {
        condition: loc(Expr::Literal(LiteralExpr::Bool(true))),
        then_branch: Box::new(Stmt::Block(vec![Stmt::Expression(loc(Expr::Literal(
            LiteralExpr::Int(1),
        )))])),
        else_branch: Some(Box::new(Stmt::Block(vec![Stmt::Expression(loc(
            Expr::Literal(LiteralExpr::Int(2)),
        ))]))),
    });

    let mut visited = Vec::new();
    let result = if_stmt.walk_post(&mut |stmt| {
        match stmt {
            Stmt::If(_) => visited.push("if"),
            Stmt::Block(_) => visited.push("block"),
            Stmt::Expression(loc_expr) => {
                if let Expr::Literal(LiteralExpr::Int(n)) = &loc_expr.node {
                    visited.push(if *n == 1 { "expr_1" } else { "expr_2" });
                }
            }
            _ => visited.push("other"),
        }
        Ok::<(), ()>(())
    });

    assert!(result.is_ok());
    assert_eq!(visited, vec!["expr_1", "block", "expr_2", "block", "if"]);
}

#[test]
fn test_find_statements() {
    // Create nested statements
    let stmt = Stmt::Block(vec![
        Stmt::VarDecl(VarDeclStmt {
            name: "x".to_string(),
            type_annotation: None,
            initializer: None,
        }),
        Stmt::If(IfStmt {
            condition: loc(Expr::Literal(LiteralExpr::Bool(true))),
            then_branch: Box::new(Stmt::VarDecl(VarDeclStmt {
                name: "y".to_string(),
                type_annotation: None,
                initializer: None,
            })),
            else_branch: None,
        }),
        Stmt::While(WhileStmt {
            condition: loc(Expr::Literal(LiteralExpr::Bool(true))),
            body: Box::new(Stmt::VarDecl(VarDeclStmt {
                name: "z".to_string(),
                type_annotation: None,
                initializer: None,
            })),
        }),
    ]);

    // Find all variable declarations
    let var_decls = stmt.find_statements(|s| matches!(s, Stmt::VarDecl(_)));
    assert_eq!(var_decls.len(), 3);

    // Extract names
    let names: Vec<&str> = var_decls
        .iter()
        .filter_map(|s| match s {
            Stmt::VarDecl(v) => Some(v.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["x", "y", "z"]);
}

#[test]
fn test_walk_expressions_in_statements() {
    // Create statements with embedded expressions
    let stmt = Stmt::Block(vec![
        Stmt::VarDecl(VarDeclStmt {
            name: "sum".to_string(),
            type_annotation: None,
            initializer: Some(loc(Expr::Binary(BinaryExpr {
                left: Box::new(loc(Expr::Identifier("a".to_string()))),
                comment_after_left: None,
                operator: BinaryOp::Add,
                comment_after_operator: None,
                right: Box::new(loc(Expr::Identifier("b".to_string()))),
            }))),
        }),
        Stmt::If(IfStmt {
            condition: loc(Expr::Identifier("c".to_string())),
            then_branch: Box::new(Stmt::Expression(loc(Expr::Identifier("d".to_string())))),
            else_branch: None,
        }),
    ]);

    let mut identifiers = Vec::new();
    let result = stmt.walk_expressions(&mut |expr| {
        if let Expr::Identifier(name) = &expr.node {
            identifiers.push(name.clone());
        }
        Ok::<(), ()>(())
    });

    assert!(result.is_ok());
    assert_eq!(identifiers, vec!["a", "b", "c", "d"]);
}

#[test]
fn test_can_exit_early() {
    // Statement without return
    let no_return = Stmt::Block(vec![
        Stmt::VarDecl(VarDeclStmt {
            name: "x".to_string(),
            type_annotation: None,
            initializer: None,
        }),
        Stmt::Expression(loc(Expr::Identifier("x".to_string()))),
    ]);
    assert!(!no_return.can_exit_early());

    // Statement with return
    let with_return = Stmt::Block(vec![
        Stmt::VarDecl(VarDeclStmt {
            name: "x".to_string(),
            type_annotation: None,
            initializer: None,
        }),
        Stmt::Return(Some(loc(Expr::Identifier("x".to_string())))),
    ]);
    assert!(with_return.can_exit_early());

    // If statement with return in one branch
    let if_with_return = Stmt::If(IfStmt {
        condition: loc(Expr::Literal(LiteralExpr::Bool(true))),
        then_branch: Box::new(Stmt::Return(Some(loc(Expr::Literal(LiteralExpr::Int(1)))))),
        else_branch: Some(Box::new(Stmt::Expression(loc(Expr::Literal(
            LiteralExpr::Int(2),
        ))))),
    });
    assert!(if_with_return.can_exit_early());
}

#[test]
fn test_nested_function_traversal() {
    // Create a function with nested statements
    let fun = Stmt::FunDecl(FunDeclStmt {
        name: "test".to_string(),
        params: vec![],
        return_type: None,
        body: Box::new(Stmt::Block(vec![
            Stmt::VarDecl(VarDeclStmt {
                name: "local".to_string(),
                type_annotation: None,
                initializer: Some(loc(Expr::Literal(LiteralExpr::Int(42)))),
            }),
            Stmt::Return(Some(loc(Expr::Identifier("local".to_string())))),
        ])),
        has_hidden_bump: false,
    });

    // Count different statement types
    let mut counts = std::collections::HashMap::new();
    let _ = fun.walk(&mut |stmt| {
        let key = match stmt {
            Stmt::FunDecl(_) => "function",
            Stmt::Block(_) => "block",
            Stmt::VarDecl(_) => "var_decl",
            Stmt::Return(_) => "return",
            _ => "other",
        };
        *counts.entry(key).or_insert(0) += 1;
        Ok::<(), ()>(())
    });

    assert_eq!(counts.get("function"), Some(&1));
    assert_eq!(counts.get("block"), Some(&1));
    assert_eq!(counts.get("var_decl"), Some(&1));
    assert_eq!(counts.get("return"), Some(&1));
}
