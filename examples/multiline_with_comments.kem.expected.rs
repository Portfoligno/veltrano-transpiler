// Test multiline function calls with comments
fn test_function(a: i64, b: i64, c: i64) -> i64 {
    return a + b + c;
}
fn main() {
    // Test 1: Comments between arguments
    let result1 = test_function(
        1,  // first argument
        2,  // second argument
        3   // third argument
    );
    // Test 2: Comments on their own lines
    let result2 = test_function(
        10,
            // This is a comment between arguments
        20,
            // Another comment
        30
    );
    // Test 3: Block comments in multiline calls
    let result3 = test_function(
        100, /* first */
        200, /* second */
        300  /* third */
    );
    // Test 4: Comments in function parameter definitions
    fn with_commented_params(x: i64  /* The x coordinate*/, y: i64  /* The y coordinate*/, z: i64   /* The z coordinate*/) -> i64 {
        return x + y + z;
    }
    let chained = Clone::clone(&result1);// Clone it
    println!("Results: {}, {}, {}", result1, result2, result3);
}
