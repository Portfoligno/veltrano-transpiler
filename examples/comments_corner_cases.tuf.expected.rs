fn greet<'a>(bump: &'a bumpalo::Bump, name: &'a str) {
    println!("Hello, {}", name);
}
fn test_function_arg_comments<'a>(bump: &'a bumpalo::Bump) {
    greet(bump, "Alice");
    greet(bump, "Bob");
}
fn test_method_chain_comments() {
    let text = "hello";
}
fn test_expression_comments() {
    let sum = 10 + 20;
    let product = 5 * 6;
    let comparison = 10 > 5;
    let grouped = 10 + 20 * 2;
}
fn test_data_structure_comments() {
}
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
fn test_brace_comments() {
    if true {
        println!("yes");
    }
    else {
        println!("no");
    }
}
fn test_multiple_comments() {
    let x = 42;
    let y = 10;
}
fn test_return_comments() -> i64 {
    return 42 + 8;
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
