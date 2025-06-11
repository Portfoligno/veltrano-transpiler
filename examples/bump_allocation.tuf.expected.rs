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

fn process_value(val__param: i64) -> i64 {
    return val__param * 2;
}
fn calculate_bonus(age: i64) -> i64 {
    return age * 100;
}
fn format_person_info<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> &'a str {
    return person.name;
}
fn get_person_name<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> &'a str {
    return person.name;
}
fn create_bump_person<'a>(bump: &'a bumpalo::Bump, name: &'a str, age: i64) -> &'a Person<'a> {
    return bump.alloc(Person { name: name, age: age });
}
fn create_bump_allocated_person<'a>(bump: &'a bumpalo::Bump, name: &'a str, age: i64) -> &'a Person<'a> {
    return bump.alloc(Person { name: name, age: age });
}
fn process_person_data<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> &'a str {
    return format_person_info(bump, person);
}
fn create_employee_record<'a>(bump: &'a bumpalo::Bump, name: &'a str, age: i64) -> &'a Person<'a> {
    let person = create_bump_allocated_person(bump, name, age);
    return person;
}
fn process_employee<'a>(bump: &'a bumpalo::Bump, person: &'a Person<'a>) -> i64 {
    let bonus = calculate_bonus(person.age);
    return bonus;
}
fn create_team<'a>(bump: &'a bumpalo::Bump, lead__name: &'a str, member__name: &'a str) -> &'a Person<'a> {
    let leader = create_bump_allocated_person(bump, lead__name, 35);
    let member = create_bump_allocated_person(bump, member__name, 28);
    return leader;
}
fn setup_office<'a>(bump: &'a bumpalo::Bump, company__name: &'a str) -> &'a Person<'a> {
    let team__lead = create_team(bump, "Alice", "Bob");
    return team__lead;
}
fn main() {
    let bump = &bumpalo::Bump::new();
    let person1 = Person { name: "Alice", age: 30 };
    let person2 = Person { name: "Bob", age: 25 };
    let company = Company { name: "TechCorp", employee__count: 50 };
    let person__ref = bump.alloc(Person { name: "Charlie", age: 35 });
    let value__ref = bump.alloc(42);
    let result1 = process_value(42);
    let result2 = process_value(person1.age);
    let bonus = calculate_bonus(person1.age);
    let formatted = format_person_info(bump, &person1);
    let processed = process_person_data(bump, &person2);
    let bump__person = create_bump_person(bump, "David", 40);
    let office__lead = setup_office(bump, company.name);
    let lead__name = get_person_name(bump, office__lead);
    let employee__bonus = process_employee(bump, office__lead);
    println!("Value operations: {} -> {}, bonus: {}", 42, result1, bonus);
    println!("Reference operations: {}, processed: {}", formatted, processed);
    println!("Transitive chain: office lead {} gets bonus {}", lead__name, employee__bonus);
    println!("Bump allocated person: {} is {} years old", bump__person.name, bump__person.age);
}
