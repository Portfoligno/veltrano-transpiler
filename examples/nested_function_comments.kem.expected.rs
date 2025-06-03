// Test nested function call comment indentation
fn f(a: i64, b: i64) -> i64 {
    return a + b;
}
fn g(x: i64, y: i64, z: i64) -> i64 {
    return x * y * z;
}
fn main() {
    // Test 1: Basic nested function calls
    let result = f(
        g(
            1,
            // Nested level comment
            2,
                // Nested level with extra indent
            3
        ),
        // Outer level comment
        4
    );
    // Test 2: Deeper nesting (3 levels)
    let deep = f(
        g(
            f(
                10,
                // Three levels deep
                20
            ),
            // Two levels deep
            30,
                    // Two levels with extra indent
            40
        ),
        50
    );
    // Test 3: Multiple nested calls
    let multi = f(
        g(1, 2, 3),  // First nested call
        g(
            4,
            // Comment in second nested call
            5,
            6
        )  // After second nested call
    );
    println!("Results: {}, {}, {}", result, deep, multi);
}
