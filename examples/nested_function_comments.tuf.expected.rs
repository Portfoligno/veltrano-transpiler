fn f(a: i64, b: i64) -> i64 {
    return a + b;
}
fn g(x: i64, y: i64, z: i64) -> i64 {
    return x * y * z;
}
fn main() {
    let result = f(
        g(
            1,
            2,
            3
        ),
        4
    );
    let deep = f(
        g(
            f(
                10,
                20
            ),
            30,
            40
        ),
        50
    );
    let multi = f(
        g(1, 2, 3),
        g(
            4,
            5,
            6
        )
    );
    println!("Results: {}, {}, {}", result, deep, multi);
}
