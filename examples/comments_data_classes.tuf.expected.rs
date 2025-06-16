#[derive(Debug, Clone)]
pub struct Person<'a> {
    pub name: &'a str,
    pub age: i64,
    pub email: &'a str,
}

fn greet<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>, formal: bool) {
    if formal {
        println!("Good day, {}", person.name);
    }
    else {
        println!("Hi, {}", person.name);
    }
}
fn main() {
    let bump = &bumpalo::Bump::new();
    let alice = Person {
        name: "Alice Smith",
        age: 30,
        email: "alice@example.com",
    };
    greet(bump, 
        &alice,
        true
    );
    let alice_name = alice.name;
    println!("Name: {}", alice_name);
}
