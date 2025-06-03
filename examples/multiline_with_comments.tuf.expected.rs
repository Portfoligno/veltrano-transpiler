fn test_function(a: i64, b: i64, c: i64) -> i64 {
    return a + b + c;
}
fn main() {
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
    let chained = Clone::clone(&result1);
    println!("Results: {}, {}, {}", result1, result2, result3);
}
