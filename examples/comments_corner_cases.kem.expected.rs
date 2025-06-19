// Test corner cases for comment preservation that currently fail or produce incorrect output
// 1. Comments inside function arguments
fn greet<'a>(bump: &'a bumpalo::Bump, name: &'a str) {
    println!("Hello, {}", name);
}
fn test_function_arg_comments<'a>(bump: &'a bumpalo::Bump) {
    // These comments break parsing or get misplaced:
    greet(bump, /* before arg */ "Alice");
    greet(bump, "Bob" /* after arg */);
    // val result = max(
    //     10, /* first arg */
    //     20  /* second arg */
    // )
}
// 2. Comments in method chains
fn test_method_chain_comments() {
    let text = "hello";
        // .toUpperCase()  // Convert to uppercase
        // .trim()         // Remove whitespace
    // This breaks the method chain:
    // val broken = "world"
    //     .toUpperCase()  // This comment breaks parsing
    //     .trim()
}
// 3. Comments inside expressions
fn test_expression_comments() {
    // Comments in binary operations
    let sum = 10 +  /* plus */ 20;
    let product = 5 *  /* times */ 6;
    // Comments in conditional expressions (if-expressions not supported)
    // Testing comments in comparison operations instead
    let comparison = (10  /* first */ >  /* greater than */ 5 /* second */);
    // Comments in parentheses
    let grouped = (10 +  /* inside parens */ 20) * 2;
}
// 4. Comments in data structures
fn test_data_structure_comments() {
    // Comments in lists (if supported)
    // val numbers = [1, /* two */ 2, 3 /* three */]
    // Comments in tuples (if supported)
    // val point = (10 /* x */, 20 /* y */)
}
// 5. Comments in function parameters
fn add(
    a: i64, /* first number */
    b: i64  /* second number */
) -> i64 {
    return a + b;
}
// 6. Comments after closing braces
fn test_brace_comments() {
    if true {
        println!("yes");
    }
    else {
        println!("no");
    } /* end else */
} /* end function */
// 7. Multiple comments on same line
fn test_multiple_comments() {
    let x = 42;  // First comment  // Second comment
    let y = 10;  /* block1 */ /* block2 */ // line comment
}
// 8. Comments in return statements
fn test_return_comments() -> i64 {
    return 42  /* the answer */ +  /* plus */ 8;
}
fn main() {
    let bump = &bumpalo::Bump::new();
    test_function_arg_comments(bump);
    test_method_chain_comments();
    test_expression_comments();
    test_brace_comments();
    test_multiple_comments();
    println!("Result: {}", test_return_comments());
}
