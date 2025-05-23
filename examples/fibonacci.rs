fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
fn main() {
    let result: i64 = fibonacci(10);
    println!(result);
}
