fn test_arithmetic_priorities() {
    let a = 2 + 3 * 4;
    let b = 20 / 4 - 2;
    let c = 10 % 3 + 1;
    let d = (2 + 3) * 4;
    let e = 20 / (4 - 2);
    let f = ((2 + 3) * (4 - 1)) / 5;
    let g = 100 / 10 / 2;
    let h = 10 - 5 - 2;
}
fn test_unary_operators() {
    let a = -5 + 3;
    let b = -((5 + 3));
    let c = -5 * 3;
    let d = -((5 * 3));
    let e = -((-10));
    let f = -((-((-5))));
    let g = -((2 + 3)) * 4;
    let h = -(((2 + 3) * 4));
}
fn test_comparison_priorities() {
    let a = 2 + 3 > 4;
    let b = 10 < 5 * 3;
    let c = (5 > 3) && (3 > 1);
    let d = 10 >= 5 + 4;
}
fn test_logical_operators() {
    let a = true || false && false;
    let b = true && false || true;
    let c = (true || false) && false;
    let d = 5 > 3 && 2 < 4;
    let e = 1 > 2 || 3 < 4 && 5 > 3;
}
fn test_method_chaining() {
    let s = "hello";
    let a = str::len(s) + 5;
    let b = String::len(&str::to_uppercase(s));
    let c = str::len(s) * 2 + 1;
}
fn test_mixed_expressions() {
    let a = -5 + 3 * 2 > 0 && true;
    let b = ((2 + 3) * 4 - 5) / (6 - 4) > 7;
    let s = "test";
    let c = str::len(s) * 2 + 3 > 10;
}
fn test_parentheses_edge_cases() {
    let a = (42);
    let b = (42);
    let c = (((42)));
    let d = -((((-42))));
    let e = (1 + 2);
    let f = (true && false);
    let g = (str::len("hello"));
    let h = (a > b);
}
fn test_commented_expressions() {
    let a = 2 * 3 + 4;
    let b = 2 + 3 * 4;
    let c = (2 + 3) * 4;
    let d = 2 * (3 + 4);
    let e = 2 + 3 * 4;
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
