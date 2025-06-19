// Parentheses Edge Cases and Comment Interactions
// Tests edge cases introduced by ParenthesizedExpr AST node
fn test_parentheses_parsing() {
    // Single value in parentheses
    let a = (42);
    let b = ((42));
    let c = (((42)));
    // Should these generate different AST nodes?
    let d = 42;  // Literal
    let e = (42);// ParenthesizedExpr containing Literal
    // Negation with parentheses
    let f = -((42));// Unary(ParenthesizedExpr(Literal))
    let g = (-42);// ParenthesizedExpr(Unary(Literal))
    let h = (-((42)));  // ParenthesizedExpr(Unary(ParenthesizedExpr(Literal)))
}
fn test_comment_preservation() {
    // Comments at different positions
    let a = (/* before expr */ 42);
    let b = (42 /* after expr */);
    let c = (/* before */ 42 /* after */);
    // Line comments force multiline
    let d = (// comment
        42
    );
    let e = (
        42
        // comment
    );
    // Mixed comment types
    let f = (/* block */
        // line
        42
    );
    let g = (// line
        42
        /* block */
    );
}
fn test_nested_parentheses_with_comments() {
    // Comments at each level
    let a = (/* outer */ (/* inner */ 42));
    let b = ((42 /* inner */) /* outer */);
    // Complex nesting
    let c = (/* 1 */ (/* 2 */ (/* 3 */ 42 /* 3 */) /* 2 */) /* 1 */);
    // Line comments in nested parens
    let d = (// outer
        (// inner
            42
        )
    );
}
fn test_binary_expression_interaction() {
    // Binary expr with parenthesized operands
    let a = (2) + (3);
    let b = (2 + 3) * (4 + 5);
    // Comments in binary with parentheses
    let c = (/* left */ 2) + (/* right */ 3);
    let d = (2 /* in left */) +  /* middle */ (3 /* in right */);
    // Line comments forcing multiline
    let e = (// left
        2
    ) + (// right
        3
    );
}
fn test_unary_with_parentheses() {
    // Various combinations
    let a = -((42));  // Unary(Parenthesized)
    let b = (-42);  // Parenthesized(Unary)
    let c = -((-((42))));   // Unary(Unary(Parenthesized))
    let d = (-((42)));// Parenthesized(Unary(Parenthesized))
    // With comments
    let e = -((/* neg */ 42));
    let f = (/* paren */ -42);
    let g = -((/* outer */ (/* inner */ 42)));
}
fn test_method_calls_in_parentheses() {
    // Method calls
    let s = "hello";
    let a = (str::len(s));
    let b = String::len(&(str::to_uppercase(s)));
    let c = (String::len(&(str::to_uppercase(s))));
    // With comments
    let d = (/* before */ str::len(s));
    // val e = (s /* obj */./* dot */ len(/* args */))  // TODO: Complex comment placement
}
fn test_precedence_ambiguities() {
    // Cases where parentheses matter
    let a1 = 2 + 3 * 4;// 14
    let a2 = (2 + 3) * 4;  // 20
    let b1 = -5 + 3;   // -2
    let b2 = -((5 + 3)); // -8
    let c1 = true || false && false;// true
    let c2 = (true || false) && false;  // false
    // Cases where parentheses don't change meaning but exist
    let d1 = 2 * 3;    // 6
    let d2 = (2 * 3);  // 6, but different AST
    let d3 = (2) * (3);// 6, even more different AST
}
fn test_whitespace_and_formatting() {
    // Various spacing
    let a = (42);
    let b = (42);
    let c = (42);
    let d = (42);
    let e = (42);
    // With operators
    let f = (2 + 3);
    let g = (2 + 3);
    let h = (2 + 3) * (4 + 5);
}
fn test_error_recovery() {
    // These might have caused issues before fixes
    // val a = (// comment without expression after - should this parse?
    // )
    // Missing closing paren - parser should recover
    // val b = (42
    // Extra closing paren - parser should recover  
    // val c = (42))
    // Valid complex case
    let d = ((((((42))))));
}
fn main() {
    println!("Testing parentheses edge cases...");
    test_parentheses_parsing();
    test_comment_preservation();
    test_nested_parentheses_with_comments();
    test_binary_expression_interaction();
    test_unary_with_parentheses();
    test_method_calls_in_parentheses();
    test_precedence_ambiguities();
    test_whitespace_and_formatting();
    // testErrorRecovery() // Commented out invalid cases
    println!("Edge case tests completed!");
}
