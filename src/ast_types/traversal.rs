//! AST traversal traits for walking and transforming expression and statement trees.
//!
//! This module provides extension traits for traversing and analyzing
//! the AST, including finding comments, expressions, and other nodes.

use super::*;

pub trait ExprExt {
    /// Walk the expression tree in pre-order
    ///
    /// Calls visitor on current node before its children. Return Err to stop early.
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>;

    /// Walk the expression tree in post-order
    ///
    /// Calls visitor on children before current node. Useful for bottom-up analysis.
    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>;

    /// Find all sub-expressions matching a predicate
    ///
    /// Returns vector of all expressions matching the predicate.
    #[allow(dead_code)]
    fn find_subexpressions<F>(&self, predicate: F) -> Vec<&LocatedExpr>
    where
        F: Fn(&LocatedExpr) -> bool;

    /// Check if any sub-expression matches a predicate
    ///
    /// Short-circuits on first match for efficiency.
    #[allow(dead_code)]
    fn any_subexpr<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool;

    /// Check if all sub-expressions match a predicate
    ///
    /// Short-circuits on first non-match for efficiency.
    #[allow(dead_code)]
    fn all_subexprs<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool;
}


impl ExprExt for LocatedExpr {
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>,
    {
        // Visit this node first (pre-order)
        visitor(self)?;

        // Then visit children
        match &self.node {
            Expr::Binary(binary) => {
                binary.left.walk(visitor)?;
                binary.right.walk(visitor)?;
            }
            Expr::Unary(unary) => {
                unary.operand.walk(visitor)?;
            }
            Expr::Call(call) => {
                call.callee.walk(visitor)?;
                for arg in &call.args {
                    match arg {
                        Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                            expr.walk(visitor)?;
                        }
                        _ => {}
                    }
                }
            }
            Expr::MethodCall(method_call) => {
                method_call.object.walk(visitor)?;
                for arg in &method_call.args {
                    arg.walk(visitor)?;
                }
            }
            Expr::FieldAccess(field_access) => {
                field_access.object.walk(visitor)?;
            }
            Expr::Literal(_) | Expr::Identifier(_) => {
                // Leaf nodes - no children to visit
            }
        }
        Ok(())
    }

    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>,
    {
        // Visit children first (post-order)
        match &self.node {
            Expr::Binary(binary) => {
                binary.left.walk_post(visitor)?;
                binary.right.walk_post(visitor)?;
            }
            Expr::Unary(unary) => {
                unary.operand.walk_post(visitor)?;
            }
            Expr::Call(call) => {
                call.callee.walk_post(visitor)?;
                for arg in &call.args {
                    match arg {
                        Argument::Bare(expr, _) | Argument::Named(_, expr, _) => {
                            expr.walk_post(visitor)?;
                        }
                        _ => {}
                    }
                }
            }
            Expr::MethodCall(method_call) => {
                method_call.object.walk_post(visitor)?;
                for arg in &method_call.args {
                    arg.walk_post(visitor)?;
                }
            }
            Expr::FieldAccess(field_access) => {
                field_access.object.walk_post(visitor)?;
            }
            Expr::Literal(_) | Expr::Identifier(_) => {
                // Leaf nodes - no children to visit
            }
        }

        // Then visit this node
        visitor(self)
    }

    fn find_subexpressions<F>(&self, predicate: F) -> Vec<&LocatedExpr>
    where
        F: Fn(&LocatedExpr) -> bool,
    {
        fn collect<'a, F>(expr: &'a LocatedExpr, predicate: &F, results: &mut Vec<&'a LocatedExpr>)
        where
            F: Fn(&LocatedExpr) -> bool,
        {
            if predicate(expr) {
                results.push(expr);
            }

            match &expr.node {
                Expr::Binary(binary) => {
                    collect(&binary.left, predicate, results);
                    collect(&binary.right, predicate, results);
                }
                Expr::Unary(unary) => {
                    collect(&unary.operand, predicate, results);
                }
                Expr::Call(call) => {
                    collect(&call.callee, predicate, results);
                    for arg in &call.args {
                        match arg {
                            Argument::Bare(e, _) | Argument::Named(_, e, _) => {
                                collect(e, predicate, results);
                            }
                            _ => {}
                        }
                    }
                }
                Expr::MethodCall(method_call) => {
                    collect(&method_call.object, predicate, results);
                    for arg in &method_call.args {
                        collect(arg, predicate, results);
                    }
                }
                Expr::FieldAccess(field_access) => {
                    collect(&field_access.object, predicate, results);
                }
                Expr::Literal(_) | Expr::Identifier(_) => {}
            }
        }

        let mut results = Vec::new();
        collect(self, &predicate, &mut results);
        results
    }

    fn any_subexpr<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool,
    {
        let mut found = false;
        let _ = self.walk(&mut |expr| {
            if predicate(expr) {
                found = true;
                Err(()) // Early exit
            } else {
                Ok(())
            }
        });
        found
    }

    fn all_subexprs<F>(&self, predicate: F) -> bool
    where
        F: Fn(&LocatedExpr) -> bool,
    {
        let mut all_match = true;
        let _ = self.walk(&mut |expr| {
            if !predicate(expr) {
                all_match = false;
                Err(()) // Early exit
            } else {
                Ok(())
            }
        });
        all_match
    }
}

/// Extension trait for statement traversal
///
/// Provides traversal methods for analyzing control flow, declarations, and nested structures.

pub trait StmtExt {
    /// Walk the statement tree in pre-order
    ///
    /// Calls visitor on each statement before its children. Return Err to stop early.
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>;

    /// Walk the statement tree in post-order
    ///
    /// Useful for bottom-up analysis or cleanup operations.
    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>;

    /// Find all sub-statements matching a predicate
    ///
    /// Recursively searches the statement tree for matching statements.
    #[allow(dead_code)]
    fn find_statements<F>(&self, predicate: F) -> Vec<&Stmt>
    where
        F: Fn(&Stmt) -> bool;

    /// Walk all expressions within this statement
    ///
    /// Visits all expressions in the statement tree for expression-level analysis.
    fn walk_expressions<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>;

    /// Check if the statement tree can exit early
    ///
    /// Returns true if the statement or any sub-statement contains a return.
    fn can_exit_early(&self) -> bool;
}


impl StmtExt for Stmt {
    fn walk<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>,
    {
        // Visit this node first (pre-order)
        visitor(self)?;

        // Then visit children
        match self {
            Stmt::Block(statements) => {
                for stmt in statements {
                    stmt.walk(visitor)?;
                }
            }
            Stmt::If(if_stmt) => {
                if_stmt.then_branch.walk(visitor)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.walk(visitor)?;
                }
            }
            Stmt::While(while_stmt) => {
                while_stmt.body.walk(visitor)?;
            }
            Stmt::FunDecl(fun_decl) => {
                fun_decl.body.walk(visitor)?;
            }
            // Leaf nodes
            Stmt::Expression(_)
            | Stmt::VarDecl(_)
            | Stmt::Return(_)
            | Stmt::Comment(_)
            | Stmt::Import(_)
            | Stmt::DataClass(_) => {}
        }
        Ok(())
    }

    fn walk_post<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&Stmt) -> Result<(), E>,
    {
        // Visit children first (post-order)
        match self {
            Stmt::Block(statements) => {
                for stmt in statements {
                    stmt.walk_post(visitor)?;
                }
            }
            Stmt::If(if_stmt) => {
                if_stmt.then_branch.walk_post(visitor)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.walk_post(visitor)?;
                }
            }
            Stmt::While(while_stmt) => {
                while_stmt.body.walk_post(visitor)?;
            }
            Stmt::FunDecl(fun_decl) => {
                fun_decl.body.walk_post(visitor)?;
            }
            // Leaf nodes
            Stmt::Expression(_)
            | Stmt::VarDecl(_)
            | Stmt::Return(_)
            | Stmt::Comment(_)
            | Stmt::Import(_)
            | Stmt::DataClass(_) => {}
        }

        // Then visit this node
        visitor(self)
    }

    fn find_statements<F>(&self, predicate: F) -> Vec<&Stmt>
    where
        F: Fn(&Stmt) -> bool,
    {
        fn collect<'a, F>(stmt: &'a Stmt, predicate: &F, results: &mut Vec<&'a Stmt>)
        where
            F: Fn(&Stmt) -> bool,
        {
            if predicate(stmt) {
                results.push(stmt);
            }

            match stmt {
                Stmt::Block(statements) => {
                    for s in statements {
                        collect(s, predicate, results);
                    }
                }
                Stmt::If(if_stmt) => {
                    collect(&if_stmt.then_branch, predicate, results);
                    if let Some(else_branch) = &if_stmt.else_branch {
                        collect(else_branch, predicate, results);
                    }
                }
                Stmt::While(while_stmt) => {
                    collect(&while_stmt.body, predicate, results);
                }
                Stmt::FunDecl(fun_decl) => {
                    collect(&fun_decl.body, predicate, results);
                }
                _ => {}
            }
        }

        let mut results = Vec::new();
        collect(self, &predicate, &mut results);
        results
    }

    fn walk_expressions<F, E>(&self, visitor: &mut F) -> Result<(), E>
    where
        F: FnMut(&LocatedExpr) -> Result<(), E>,
    {
        match self {
            Stmt::Expression(expr) => expr.walk(visitor)?,
            Stmt::VarDecl(var_decl) => {
                if let Some(init) = &var_decl.initializer {
                    init.walk(visitor)?;
                }
            }
            Stmt::Return(Some(expr)) => expr.walk(visitor)?,
            Stmt::If(if_stmt) => {
                if_stmt.condition.walk(visitor)?;
                if_stmt.then_branch.walk_expressions(visitor)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.walk_expressions(visitor)?;
                }
            }
            Stmt::While(while_stmt) => {
                while_stmt.condition.walk(visitor)?;
                while_stmt.body.walk_expressions(visitor)?;
            }
            Stmt::Block(statements) => {
                for stmt in statements {
                    stmt.walk_expressions(visitor)?;
                }
            }
            Stmt::FunDecl(fun_decl) => {
                fun_decl.body.walk_expressions(visitor)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn can_exit_early(&self) -> bool {
        match self {
            Stmt::Return(_) => true,
            Stmt::Block(statements) => statements.iter().any(|s| s.can_exit_early()),
            Stmt::If(if_stmt) => {
                if_stmt.then_branch.can_exit_early()
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |s| s.can_exit_early())
            }
            Stmt::While(while_stmt) => while_stmt.body.can_exit_early(),
            Stmt::FunDecl(fun_decl) => fun_decl.body.can_exit_early(),
            _ => false,
        }
    }
}
