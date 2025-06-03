fn main() {
    let immutable_var: &str = "Hello";
    println!("{}", immutable_var);
    let inferred = "World";
    println!("{}", inferred);
    let number: i64 = 42;
    let flag: bool = true;
    let empty: () = ();
    println!("Number: {}", number);
    println!("Flag: {}", flag);
}
