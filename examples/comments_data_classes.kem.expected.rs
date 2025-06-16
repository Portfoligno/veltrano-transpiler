// Test comment handling in data classes and type annotations
// Data class with field comments
#[derive(Debug, Clone)]
pub struct Person<'a> {
    pub name: &'a str,  // Person's full name
    pub age: i64,   // Age in years
    pub email: &'a str,  // Contact email
}

// Function with parameter comments
fn greet<'a>(
    bump: &'a bumpalo::Bump,
    person: &'a Person<'a>,  // The person to greet
    formal: bool // Whether to use formal greeting
) {
    if formal {
        println!("Good day, {}", person.name);
    }
    else {
        println!("Hi, {}", person.name);
    }
}
// Test various comment positions
fn main() {
    let bump = &bumpalo::Bump::new();
    // Create a person instance
    let alice = Person {
        name: "Alice Smith",  // Full name
        age: 30,      // Age
        email: "alice@example.com",  // Email address
    };
    // Test function calls with comments
    greet(bump, 
        &alice,  // Pass person by reference
        true  // Use formal greeting
    );
    // Test field access
    let alice_name = alice.name;  // Extract name field
    println!("Name: {}", alice_name);
}
