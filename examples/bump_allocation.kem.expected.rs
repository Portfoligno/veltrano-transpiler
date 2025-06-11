// Cross-scope reference preparation using automatic bump parameter detection
// Shows how functions prepare references for use in caller scope
#[derive(Debug, Clone)]
pub struct Person<'a> {
    pub name: &'a str,
    pub age: i64,
}

#[derive(Debug, Clone)]
pub struct Company<'a> {
    pub name: &'a str,
    pub employee__count: i64,
}

// ========= VALUE-ONLY FUNCTIONS (NO bump parameters) =========
fn process_value(val__param: i64) -> i64 {
    return val__param * 2;
}
fn calculate_bonus(age: i64) -> i64 {
    return age * 100;
}
// ========= REFERENCE-HANDLING FUNCTIONS (GET bump parameters) =========
// Function with reference types - GETS automatic bump parameter
fn format_person_info<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> &'a str {
    return person.name;
}
fn get_person_name<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> &'a str {
    return person.name;
}
// ========= EXPLICIT BUMP ALLOCATION (GET bump parameters) =========
// Function with explicit bump allocation - GETS automatic bump parameter
fn create_bump_person<'a>(bump: &'a bumpalo::Bump, name: &'a str, age: i64) -> &'a Person<'a> {
    return bump.alloc(Person { name: name, age: age });
}
fn create_bump_allocated_person<'a>(bump: &'a bumpalo::Bump, name: &'a str, age: i64) -> &'a Person<'a> {
    return bump.alloc(Person { name: name, age: age });
}
// ========= TRANSITIVE BUMP FUNCTIONS (GET bump parameters) =========
// Function calling bump-requiring function - GETS automatic bump parameter transitively
fn process_person_data<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> &'a str {
    return format_person_info(bump, person);
}
fn create_employee_record<'a>(bump: &'a bumpalo::Bump, name: &'a str, age: i64) -> &'a Person<'a> {
    let person = create_bump_allocated_person(bump, name, age);
    return person;
}
// Function that mixes value types and references
fn process_employee<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> i64 {
    let bonus = calculate_bonus(person.age);
      // No bump needed for calculateBonus
    return bonus;
}
// Complex transitive chain: main -> setupOffice -> createTeam -> createBumpAllocatedPerson
fn create_team<'a>(bump: &'a bumpalo::Bump, lead__name: &'a str, member__name: &'a str) -> &'a Person<'a> {
    let leader = create_bump_allocated_person(bump, lead__name, 35);
    let member = create_bump_allocated_person(bump, member__name, 28);
    return leader;
      // Return the leader for this example
}
fn setup_office<'a>(bump: &'a bumpalo::Bump, company__name: &'a str) -> &'a Person<'a> {
    let team__lead = create_team(bump, "Alice", "Bob");
    return team__lead;
}
fn main() {
    let bump = &bumpalo::Bump::new();
    // Regular allocation - uses standard heap
    let person1 = Person { name: "Alice", age: 30 };
    let person2 = Person { name: "Bob", age: 25 };
    let company = Company { name: "TechCorp", employee__count: 50 };
    // Explicit bump allocation using .bumpRef()
    let person__ref = bump.alloc(Person { name: "Charlie", age: 35 });
    let value__ref = bump.alloc(42);
    // Function calls demonstrate automatic bump parameter behavior:
    let result1 = process_value(42);
           // No bump needed (value types only)
    let result2 = process_value(person1.age);
      // No bump needed (extracts value type)
    let bonus = calculate_bonus(person1.age);
      // No bump needed (value types only)
    let formatted = format_person_info(bump, &person1);
     // Automatic bump (handles reference types)
    let processed = process_person_data(bump, &person2);
     // Automatic bump (transitively calls formatPersonInfo)
    let bump__person = create_bump_person(bump, "David", 40);
     // Automatic bump (explicit .bumpRef())
    // Transitive analysis - this triggers the entire chain
    let office__lead = setup_office(bump, company.name);
    let lead__name = get_person_name(bump, office__lead);
    let employee__bonus = process_employee(bump, office__lead);
    println!("Value operations: {} -> {}, bonus: {}", 42, result1, bonus);
    println!("Reference operations: {}, processed: {}", formatted, processed);
    println!("Transitive chain: office lead {} gets bonus {}", lead__name, employee__bonus);
    println!("Bump allocated person: {} is {} years old", bump__person.name, bump__person.age);
}
// CROSS-SCOPE REFERENCE PREPARATION:
//
// This demonstrates the core value: functions can prepare references that remain
// valid in the caller's scope without manual lifetime management.
//
// Functions that prepare references:
// - formatPersonInfo, getPersonName: Return string references from Person data
// - createBumpPerson, createBumpAllocatedPerson: Create Person instances with prepared references
// - processPersonData, createEmployeeRecord: Build results using reference preparation
// - createTeam, setupOffice: Factory functions that prepare complex structures
// - main: Uses all the prepared references without managing allocator details
//
// Functions that don't prepare references:
// - processValue, calculateBonus: Pure value operations (Int -> Int)
//
// Key benefit: Clean separation between reference preparation (in callees) and
// reference usage (in callers), with automatic lifetime management.
