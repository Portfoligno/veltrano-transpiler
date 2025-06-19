fn test_comment_types() {
    let a = 42;
    let b = 42;
    let c = 42;
    let d = 42;
    let e = 42;
    let f = 42;
}
fn test_comments_in_operator_chains() {
    let a = 1 + 2 * 3 - 4;
    let b = 1 + 2 * 3 - 4;
    let c = 1 + 2 * 3;
}
fn test_comments_and_precedence() {
    let a = 2 + 3 * 4;
    let b = 2 * 3 + 4;
    let c = (2 + 3) * 4;
    let d = 2 + (3 * 4);
}
fn test_adjacent_comments() {
    let a = 42;
    let b = 42;
}
fn test_comments_in_function_calls() {
    foo(1, 2);
    foo(
        3,
        4
    );
    foo(
        1,
        2
    );
    bar();
}
fn test_comments_in_method_chains() {
    let s = "hello";
}
fn test_edge_positions() {
    let a = 42;
    let b = 42;
}
fn test_comment_content() {
    let a = 42;
    let b = 42;
    let c = 42;
    let d = 42;
    let e = 42;
    let f = 42;
}
fn test_whitespace_interaction() {
    let a = 42 + 5;
    let b = 42 + 5;
    let c = 42;
}
fn foo(a: i64, b: i64) {
    println!("{} {}", a, b);
}
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
