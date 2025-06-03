#[derive(Debug, Clone)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone)]
pub struct Person<'a> {
    pub name: &'a str,
    pub age: i64,
}

#[derive(Debug, Clone)]
pub struct Book<'a> {
    pub title: &'a str,
    pub author: &'a Person<'a>,
    pub pages: i64,
}

#[derive(Debug, Clone)]
pub struct Address<'a> {
    pub street: &'a str,
    pub city: &'a str,
    pub zip_code: i64,
}

#[derive(Debug, Clone)]
pub struct Company<'a> {
    pub name: &'a str,
    pub address: &'a Address<'a>,
    pub employees: i64,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let alice = Person { name: "Alice", age: 30 };
    let book = Book { title: "The Rust Book", author: &alice, pages: 500 };
    let x = 100;
    let y = 200;
    let p2 = Point { x, y };
    let name = "Bob";
    let bob = Person { name, age: 25 };
    let charlie = Person { age: 40, name: "Charlie" };
    let book2 = Book { pages: 300, author: &alice, title: "Rust in Action" };
    let david = Person { name, age: 45 };
    let emma = Person { age: 35, name };
    let title = "Learning Rust";
    let pages = 250;
    let book3 = Book { title, author: &alice, pages: 400 };
    let book4 = Book { title: "Rust Guide", author: &alice, pages };
    let alice_name = alice.name;
    let alice_age = alice.age;
    let book_title = book.title;
    println!("Data classes compiled successfully!");
    println!("Alice: {} is {} years old", alice_name, alice_age);
    println!("Book: {}", book_title);
}
