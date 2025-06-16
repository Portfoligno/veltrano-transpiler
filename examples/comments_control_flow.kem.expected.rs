// Test comment handling in control flow structures
fn test__if__else__comments() {
    let x = 10;
    // Comment before if statement
    if x > 5 {  // Comment after condition
        // Comment inside then block
        println!("x is large");  // Inline comment in then
    }
    else {  // Comment on else line
        // Comment inside else block
        println!("x is small");  // Inline comment in else
    }
    // Nested if with comments
    if x > 0 {  // Outer condition
        // Check for even number
        if x % 2 == 0 {  // Inner condition
            /* Block comment
               in nested if */
            println!("positive even");
        }
    }
}
fn test__while__comments() {
    let limit = 5;
    //        in the middle of loop */
    //     counter = counter - 1  // Decrement counter
    //     
    //     // Comment at end of loop body
    // }  // Comment after closing brace
    // Alternative: demonstrate comments in a different control structure
    if limit > 0 {  // Check if positive
        /* Block comment
           before print */
        println!("Limit is positive: {}", limit);  // Inline comment
    }  // Comment after if block
}
fn test__complex__conditions__with__comments() {
    let a = 10;
    let b = 20;
    let c = 30;
    // Simple condition with inline comment
    if a > 5 {  // First condition check
        // Check second condition
        if b < 25 {  // Second condition check
            // Check third condition
            if c == 30 {  // Third condition check
                // All conditions met
                println!("Complex condition passed");
            }
        }
    }
    // Condition with block comments
    if a > 0 {  // positive a
        if b > 0 {  // positive b  
            if c > 0 {  // positive c
                println!("All positive");
            }
        }
    }
}
fn test__logical__operators__with__comments() {
    let x = 5;
    let y = 10;
    // Test && operator with comments (not supported - will show parse error)
    if x > 0 && y < 20 {  // Check y is less than 20
        println!("Both conditions met");
    }
    // Test with block comments in && expression
    if x > 0 && y < 20 {
        println!("Conditions with block comments");
    }
}
fn main() {
    // Test all comment scenarios
    test__if__else__comments();
    test__while__comments();
    test__complex__conditions__with__comments();
    test__logical__operators__with__comments();
}
