fn test_function(a: i64, b: i64, c: i64) -> i64 {
    return a + b + c;
}
fn main() {
    let bump = &bumpalo::Bump::new();
    let result1 = test_function(
        1,
        2,
        3
    );
    let result2 = test_function(
        10,
        20,
        30
    );
    let result3 = test_function(
        100,
        200,
        300
    );
    fn with_commented_params(x: i64, y: i64, z: i64) -> i64 {
        return x + y + z;
    }
    let result5 = test_function(
        50,
        60,
        70
    );
    let chained = Clone::clone(&result1);
    let message: &&str = bump.alloc("Hello");
    let chained2: &&&&&str = bump.alloc(&&message);
    let mixed = bump.alloc(&&message);
    let block_chained = Clone::clone(&result1);
    let mixed_comments = bump.alloc(&&message);
    println!("Results: {}, {}, {}, {}", result1, result2, result3, result5);
}
