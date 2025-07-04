// Test multiline function calls with comments
fn test_function(a: i64, b: i64, c: i64) -> i64 {
    return a + b + c;
}
fn main() {
    let bump = &bumpalo::Bump::new();
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
        100,
         /* first */ 200,
         /* second */ 300  /* third */
    );
    // Test 4: Comments in function parameter definitions
    fn with_commented_params(
        x: i64,  // The x coordinate
        y: i64,  // The y coordinate
        z: i64   // The z coordinate
    ) -> i64 {
        return x + y + z;
    }
    // Test 5: Extra indented standalone comments
    let result5 = test_function(
        50,
            // This comment has 4 extra spaces
        60,
                // This comment has 8 extra spaces
        70
    );
    // Test 6: Comments in method chains
    let chained = Clone::clone(&result1);  // Get reference // Clone it
    // Test 7: Method chain that won't be optimized away
    let message: &&str = bump.alloc("Hello");
    let chained2: &&&&&str = bump.alloc(&&message);  // First ref // Second ref // Final bumpRef
    // Test 8: Mixed style method chain with comments
    let mixed = bump.alloc(&&message);  // Start on same line // Continue on next // And finish
    // Test 9: Block comments in method chains
    let block_chained = Clone::clone(&result1);  /* Get reference (block) */ /* Clone it (block) */
    // Test 10: Mixed block and line comments in method chains
    let mixed_comments = bump.alloc(&&message);  /* First block comment */ // Line comment in middle /* Final block comment */
    println!("Results: {}, {}, {}, {}", result1, result2, result3, result5);
}
