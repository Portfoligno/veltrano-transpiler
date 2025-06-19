//! Integration tests for the modularized parser
//!
//! These tests verify that the parser modules work correctly together
//! after the refactoring into separate modules.

use veltrano::ast::*;
use veltrano::config::Config;
use veltrano::lexer::Lexer;
use veltrano::parser::Parser;

/// Helper function to parse source code
fn parse(source: &str) -> Result<Program, veltrano::error::VeltranoError> {
    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Helper function to parse with comment preservation
fn parse_with_comments(source: &str) -> Result<Program, veltrano::error::VeltranoError> {
    let config = Config {
        preserve_comments: true,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_expression_parsing_module() {
    // Test that expression parsing works correctly
    let source = r#"
        1 + 2 * 3
        x == y && z
        obj.field
        func(a, b, c)
        arr.map(double)
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 5);

    // All should be expression statements
    for stmt in &program.statements {
        assert!(matches!(stmt, Stmt::Expression(_)));
    }
}

#[test]
fn test_statement_parsing_module() {
    // Test that statement parsing works correctly
    let source = r#"
        val x = 10
        fun add(a: I32, b: I32): I32 {
            a + b
        }
        if (x > 5) {
            println("big")
        } else {
            println("small")
        }
        while (x > 0) {
            println("loop")
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error in statement test: {}", e);
    }
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 4);

    assert!(matches!(&program.statements[0], Stmt::VarDecl(_)));
    assert!(matches!(&program.statements[1], Stmt::FunDecl(_)));
    assert!(matches!(&program.statements[2], Stmt::If(_)));
    assert!(matches!(&program.statements[3], Stmt::While(_)));
}

#[test]
fn test_type_parsing_module() {
    // Test that type parsing works correctly
    let source = r#"
        val a: I32 = 5
        val b: String = "hello"
        val c: Vec<I32> = vec()
        val d: Option<String> = None
        val e: Result<I32, String> = Ok(42)
        val f: Array<Bool, 10> = array()
        val g: Ref<String> = "test"
        val h: MutRef<I32> = x
    "#;

    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 8);

    // All should be variable declarations with type annotations
    for stmt in &program.statements {
        match stmt {
            Stmt::VarDecl(var_decl) => {
                assert!(var_decl.type_annotation.is_some());
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}

#[test]
fn test_error_recovery() {
    // Test that error recovery works across modules
    let source = r#"
        val x = 10
        val y = // missing initializer
        val z = 20
        fun test() {
            invalid syntax here
        }
        val w = 30
    "#;

    let config = Config {
        preserve_comments: false,
    };
    let mut lexer = Lexer::with_config(source.to_string(), config);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, errors) = parser.parse_with_recovery();

    // Should have recovered and parsed some statements
    assert!(program.statements.len() >= 3);
    assert!(!errors.errors().is_empty());

    // Check that we parsed the valid declarations
    let var_count = program
        .statements
        .iter()
        .filter(|s| matches!(s, Stmt::VarDecl(_)))
        .count();
    assert!(var_count >= 3); // x, z, and w should be parsed
}

#[test]
fn test_comment_preservation() {
    // Test that comments are preserved correctly
    let source = r#"
        // Function comment
        fun test(
            x: I32, // parameter comment
            y: String
        ): Bool {
            // Body comment
            true
        }
        
        /* Block comment */
        val z = 42 // inline comment
    "#;

    let result = parse_with_comments(source);
    assert!(result.is_ok());
    let program = result.unwrap();

    // Should have comments as separate statements
    let comment_count = program
        .statements
        .iter()
        .filter(|s| matches!(s, Stmt::Comment(_)))
        .count();
    assert!(comment_count > 0);
}

#[test]
fn test_method_chaining_across_lines() {
    // Test that method chaining works correctly with newlines
    let source = r#"
        result
            .map(double)
            .filter(isLarge)
            .collect()
    "#;

    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    // Should be a single expression statement with chained method calls
    match &program.statements[0] {
        Stmt::Expression(expr) => {
            // The outermost should be a method call (collect)
            assert!(matches!(&expr.node, Expr::MethodCall(_)));
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_complex_function_calls() {
    // Test parsing of complex function calls with various argument types
    let source = r#"
        func(
            42,
            name = "test",
            .field,
            nested(1, 2),
            inc
        )
    "#;

    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Expression(expr) => {
            match &expr.node {
                Expr::Call(call) => {
                    assert_eq!(call.args.len(), 5);
                    assert!(call.is_multiline);

                    // Check argument types
                    assert!(matches!(&call.args[0], Argument::Bare(_, _)));
                    assert!(matches!(&call.args[1], Argument::Named(_, _, _)));
                    assert!(matches!(&call.args[2], Argument::Shorthand(_, _)));
                }
                _ => panic!("Expected function call"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_data_class_parsing() {
    // Test data class parsing
    let source = r#"
        data class Point(val x: I32, val y: I32)
        data class Person(
            val name: String,
            val age: I32,
            val email: Option<String>
        )
    "#;

    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);

    match &program.statements[0] {
        Stmt::DataClass(dc) => {
            assert_eq!(dc.name, "Point");
            assert_eq!(dc.fields.len(), 2);
        }
        _ => panic!("Expected data class"),
    }

    match &program.statements[1] {
        Stmt::DataClass(dc) => {
            assert_eq!(dc.name, "Person");
            assert_eq!(dc.fields.len(), 3);
        }
        _ => panic!("Expected data class"),
    }
}

#[test]
fn test_import_statements() {
    // Test import statement parsing
    let source = r#"
        import String.length
        import Vec.push as vecPush
        import Option.map
    "#;

    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 3);

    match &program.statements[0] {
        Stmt::Import(import) => {
            assert_eq!(import.type_name, "String");
            assert_eq!(import.method_name, "length");
            assert!(import.alias.is_none());
        }
        _ => panic!("Expected import"),
    }

    match &program.statements[1] {
        Stmt::Import(import) => {
            assert_eq!(import.type_name, "Vec");
            assert_eq!(import.method_name, "push");
            assert_eq!(import.alias, Some("vecPush".to_string()));
        }
        _ => panic!("Expected import"),
    }
}

#[test]
fn test_double_minus_error() {
    // Test that double minus is caught as an error
    let source = "val x = --5";

    let result = parse(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_string = format!("{}", err);
    assert!(err_string.contains("Double minus"));
}

#[test]
fn test_nested_expressions() {
    // Test deeply nested expressions
    let source = r#"
        ((a + b) * (c - d)) / (e % f)
        obj.field.method(x, y).result
        result.unwrap()
    "#;

    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 3);
}

#[test]
fn test_bump_allocation_analysis() {
    // Test that bump allocation analysis works correctly
    let source = r#"
        fun outer(): Vec<String> {
            vec()
        }
        
        fun inner() {
            val v = outer()
            v.push("test")
        }
    "#;

    let result = parse(source);
    if let Err(e) = &result {
        eprintln!("Parse error in bump test: {}", e);
    }
    assert!(result.is_ok());
    let program = result.unwrap();

    // Check that bump allocation flags are set correctly
    for stmt in &program.statements {
        if let Stmt::FunDecl(fun_decl) = stmt {
            match fun_decl.name.as_str() {
                "outer" => assert!(fun_decl.has_hidden_bump),
                "inner" => assert!(fun_decl.has_hidden_bump),
                _ => {}
            }
        }
    }
}
