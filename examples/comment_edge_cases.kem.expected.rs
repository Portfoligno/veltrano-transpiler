// Comment Edge Cases in Expressions
// Tests various comment scenarios that might break parsing or formatting
fn test_comment_types() {
    // Single line comment formats
    let a = 42; // comment at end
    let b = 42;
    let c = 42; /* after */ // and line comment
    // Multi-line block comments
    let d = 42;
    // Nested-looking comments (not actually nested)
    let e = 42;
    let f = 42;
}
fn test_comments_in_operator_chains() {
    // Between each operator
    let a = 1 +  /* c1 */ 2 *  /* c2 */ 3 -  /* c3 */ 4;
    // Line comments forcing newlines
    let b = 1 +  // plus
    2 *  // times
    3 -  // minus
    4;
    // Mixed line and block
    let c = 1  /* block */ +  // line
    2 *  /* another block */ 3; // end line
}
fn test_comments_and_precedence() {
    // Comments shouldn't affect parsing precedence
    let a = 2 +  /* higher precedence next */ 3 * 4;
    let b = 2 * 3  /* lower precedence next */ + 4;
    // With parentheses
    let c = (2 + 3)  /* now multiply */ * 4;
    let d = 2 +  /* parentheses change precedence */ (3 * 4);
}
fn test_adjacent_comments() {
    // Multiple comments in sequence
    let a = 42;
    let b = 42; /* c1 */ /* c2 */ /* c3 */
    // Line comments in sequence
    // TODO: Parser doesn't support line comment with newline before expression
    // val c = // comment 1
    //         // comment 2
    //         // comment 3
    //         42
    // Mixed adjacent
    // TODO: Parser doesn't support line comment at end of line in expressions
    // val d = /* block */ // line
    //         42
    // val e = // line
    //         /* block */ 42
}
fn test_comments_in_function_calls() {
    // Every possible position
    foo(/* before first */ 1 /* after first */, /* before second */ 2 /* after second */);
    // With newlines
    foo(
        /* before first */ 3, /* after first */
        /* before second */ 4 /* after second */
    );
    // Line comments
    // TODO: Parser in preserve-comments mode doesn't handle line comments before arguments
    foo(
        1, // after first
        2  // after second
    );
    // Empty-looking calls
    bar();  // Normal call works
    // TODO: Parser in preserve-comments mode doesn't handle empty args with only comments
    // bar(/* just comments */)
    // bar(
    //     // just a comment
    // )
}
fn test_comments_in_method_chains() {
    let s = "hello";
    // Between chain elements
    // val a = s /* c1 */ . /* c2 */ len /* c3 */ ()  // TODO: Parser doesn't support spaces after dot
    // With line comments and newlines
    // TODO: Parser doesn't support newlines after dot operator
    // val b = s // object
    //         . // dot
    //         toUppercase // method
    //         () // call
    //         . // chain
    //         len // next method
    //         () // final call
    // Complex chain
    // TODO: Parser doesn't support comments/newlines around dot operator
    // val c = s
    //         /* block before dot */ . /* block after dot */
    //         // line comment
    //         toUppercase()
    //         /* between calls */
    //         .len()
}
fn test_edge_positions() {
    // Start of file handled separately
    // Before statement
    /* comment */
    let a = 42;
    // In weird places
    let b = 42; /* before semicolon (implicit) */
    // Multiple on same line
    // /* c1 */ val c /* c2 */ = /* c3 */ 42 /* c4 */ // c5  // TODO: Comment between name and =
    // Between keywords
    // val /* between val and name */ d = 42  // TODO: Comment between val and name
}
fn test_comment_content() {
    // Special characters in comments
    let a = 42; /* comment with "quotes" and 'apostrophes' */
    let b = 42; /* comment with \ backslash and / slash */
    let c = 42; /* comment with newline \n and tab \t */
    // Code-like content
    let d = 42; /* val fake = 99; this isn't real code */
    let e = 42; // } this brace doesn't close anything
    // Very long comments
    let f = 42; /* This is a very long comment that goes on and on and on and might wrap in some editors or displays but should still be handled correctly by the parser and code generator */
}
fn test_whitespace_interaction() {
    // No space around comments
    let a = 42 /* comment */ + /* comment */ 5;
    // Lots of space
    let b = 42 /*  comment  */ + /*  comment  */ 5;
    // Tabs and spaces
    let c = 42;	/* tab before */  /* spaces before */
    // Newlines and comments
    // TODO: Parser doesn't support line continuation with comments
    // val d = 42 /* comment */
    //         /* another comment */
    //         + 5
}
// This is a utility function for testing
fn foo(a: i64, b: i64) {
    println!("{} {}", a, b);
}
// Zero-argument function for testing empty calls with comments
fn bar() {
    println!("bar called");
}
fn main() {
    println!("Testing comment edge cases...");
    test_comment_types();
    test_comments_in_operator_chains();
    test_comments_and_precedence();
    test_adjacent_comments();
    test_comments_in_function_calls();
    test_comments_in_method_chains();
    test_edge_positions();
    test_comment_content();
    test_whitespace_interaction();
    println!("Comment edge case tests completed!");
}
