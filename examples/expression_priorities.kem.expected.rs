// Expression Priorities and Edge Cases Test
// This file tests various expression constructs and their priorities
fn test_arithmetic_priorities() {
    // Basic arithmetic precedence
    let a = 2 + 3 * 4;  // Should be 2 + (3 * 4) = 14
    let b = 20 / 4 - 2;  // Should be (20 / 4) - 2 = 3
    let c = 10 % 3 + 1;  // Should be (10 % 3) + 1 = 2
    // Parentheses override precedence
    let d = (2 + 3) * 4;  // Should be 20
    let e = 20 / (4 - 2);  // Should be 10
    // Nested parentheses
    let f = ((2 + 3) * (4 - 1)) / 5;  // Should be (5 * 3) / 5 = 3
    // Multiple operators same precedence (left-to-right)
    let g = 100 / 10 / 2;  // Should be (100 / 10) / 2 = 5
    let h = 10 - 5 - 2;  // Should be (10 - 5) - 2 = 3
}
fn test_unary_operators() {
    // Unary minus precedence
    let a = -5 + 3;  // Should be (-5) + 3 = -2
    let b = -((5 + 3));  // Should be -(8) = -8
    let c = -5 * 3;  // Should be (-5) * 3 = -15
    let d = -((5 * 3));  // Should be -(15) = -15 (same result)
    // Double negation
    let e = -((-10));  // Should be 10
    let f = -((-((-5))));  // Should be -5
    // Unary with parentheses
    let g = -((2 + 3)) * 4;  // Should be (-5) * 4 = -20
    let h = -(((2 + 3) * 4));  // Should be -(20) = -20
}
fn test_comparison_priorities() {
    // Comparison vs arithmetic
    let a = 2 + 3 > 4;  // Should be (2 + 3) > 4 = true
    let b = 10 < 5 * 3;  // Should be 10 < (5 * 3) = true
    // Multiple comparisons (not chainable in most languages)
    let c = (5 > 3) && (3 > 1);  // Explicit grouping
    let d = 10 >= 5 + 4;  // Should be 10 >= (5 + 4) = true
}
fn test_logical_operators() {
    // AND has higher precedence than OR
    let a = true || false && false;  // Should be true || (false && false) = true
    let b = true && false || true;  // Should be (true && false) || true = true
    // Parentheses to override
    let c = (true || false) && false;  // Should be true && false = false
    // With comparisons
    let d = 5 > 3 && 2 < 4;  // Should be (5 > 3) && (2 < 4) = true
    let e = 1 > 2 || 3 < 4 && 5 > 3;  // Should be (1 > 2) || ((3 < 4) && (5 > 3)) = true
}
fn test_method_chaining() {
    // Method calls bind tightly
    let s = "hello";
    let a = str::len(s) + 5;  // Should be (s.len()) + 5
    let b = String::len(&str::to_uppercase(s));  // Chained calls
    // With arithmetic
    let c = str::len(s) * 2 + 1;  // Should be ((s.len()) * 2) + 1
}
fn test_mixed_expressions() {
    // Complex mixed expression
    let a = -5 + 3 * 2 > 0 && true;  // Should be (((-5) + (3 * 2)) > 0) && true = true
    // Very nested
    let b = ((2 + 3) * 4 - 5) / (6 - 4) > 7;  // Should be ((20 - 5) / 2) > 7 = true
    // Method calls in arithmetic
    let s = "test";
    let c = str::len(s) * 2 + 3 > 10;  // Should be (((s.len()) * 2) + 3) > 10 = true
}
fn test_parentheses_edge_cases() {
    // Empty-looking parentheses with comments
    let a = (/* comment */ 42);
    let b = (// line comment
        42
    );
    // Multiple parentheses
    let c = (((42)));  // Should still be 42
    let d = -((((-42))));  // Should be 42
    // Parentheses with all expression types
    let e = (1 + 2);
    let f = (true && false);
    let g = (str::len("hello"));
    let h = (a > b);
}
fn test_commented_expressions() {
    // Comments shouldn't affect precedence
    let a = 2  /* multiply */ * 3  /* add */ + 4;  // Should be (2 * 3) + 4 = 10
    let b = 2 +  /* comment */ 3 *  /* comment */ 4;  // Should be 2 + (3 * 4) = 14
    // Comments in parentheses
    let c = (/* start */ 2 + 3 /* end */) * 4;  // Should be 20
    let d = 2 * (3  /* inside */ + 4);  // Should be 14
    // Line comments with expressions
    let e = 2 +  // first part
    3 *  // second part  
    4;   // Should be 14
}
fn main() {
    println!("Testing expression priorities...");
    test_arithmetic_priorities();
    test_unary_operators();
    test_comparison_priorities();
    test_logical_operators();
    test_method_chaining();
    test_mixed_expressions();
    test_parentheses_edge_cases();
    test_commented_expressions();
    println!("All tests completed!");
}
