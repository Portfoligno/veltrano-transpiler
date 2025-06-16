fn test__if__else__comments() {
    let x = 10;
    if x > 5 {
        println!("x is large");
    }
    else {
        println!("x is small");
    }
    if x > 0 {
        if x % 2 == 0 {
            println!("positive even");
        }
    }
}
fn test__while__comments() {
    let limit = 5;
    if limit > 0 {
        println!("Limit is positive: {}", limit);
    }
}
fn test__complex__conditions__with__comments() {
    let a = 10;
    let b = 20;
    let c = 30;
    if a > 5 {
        if b < 25 {
            if c == 30 {
                println!("Complex condition passed");
            }
        }
    }
    if a > 0 {
        if b > 0 {
            if c > 0 {
                println!("All positive");
            }
        }
    }
}
fn test__logical__operators__with__comments() {
    let x = 5;
    let y = 10;
    if x > 0 && y < 20 {
        println!("Both conditions met");
    }
    if x > 0 && y < 20 {
        println!("Conditions with block comments");
    }
}
fn main() {
    test__if__else__comments();
    test__while__comments();
    test__complex__conditions__with__comments();
    test__logical__operators__with__comments();
}
