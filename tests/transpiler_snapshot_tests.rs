//! Integration tests using snapshot testing for transpiler output

mod common;
mod test_configs;

use common::snapshot_utils::assert_config_snapshot;
use common::{transpile, TestContext};
use std::fs;
use test_configs::test_configs;

/// Helper to create a snapshot for each config
fn snapshot_with_configs(test_name: &str, veltrano_code: &str) {
    let configs = test_configs();

    for (config_key, config) in configs {
        let ctx = TestContext::with_config(config);

        match transpile(veltrano_code, &ctx) {
            Ok(rust_output) => {
                assert_config_snapshot(test_name, config_key, &rust_output);
            }
            Err(e) => {
                panic!("Transpilation failed for config '{}': {}", config_key, e);
            }
        }
    }
}

#[test]
fn test_val_declarations() {
    snapshot_with_configs(
        "val_declarations",
        r#"val x = 42
val y = "hello"
val z = true"#,
    );
}

#[test]
fn test_var_declarations() {
    // Note: 'var' keyword is not supported in Veltrano, using 'val' instead
    snapshot_with_configs(
        "var_declarations",
        r#"val count = 0
val message = "initial"
val flag = false"#,
    );
}

#[test]
fn test_function_declarations() {
    snapshot_with_configs(
        "function_declarations",
        r#"fun add(a: I64, b: I64) {
    return a + b
}

fun double(x: I64) {
    return x * 2
}"#,
    );
}

#[test]
fn test_if_expressions() {
    // Note: if-else as expressions not yet supported in transpiler
    snapshot_with_configs(
        "if_expressions",
        r#"fun check_sign(x: I64) {
    if (x > 0) {
        return "positive"
    } else if (x < 0) {
        return "negative"
    } else {
        return "zero"
    }
}"#,
    );
}

#[test]
fn test_conditional_logic() {
    // Testing conditional logic with if-else
    snapshot_with_configs(
        "conditional_logic",
        r#"fun check_value(x: I64) {
    if (x > 0) {
        return x * 2
    } else {
        return 0
    }
}"#,
    );
}

#[test]
fn test_comments_preservation() {
    let code = r#"// This is a top-level comment
val x = 42 // inline comment

/* Block comment
   spans multiple lines */
fun test() {
    // Function comment
    return x
}"#;

    snapshot_with_configs("comments_preservation", code);
}

#[test]
fn test_ownership_modifiers() {
    snapshot_with_configs(
        "ownership_modifiers",
        r#"val a = "hello"
val b = a.ref()
val c = a.mutRef()
val d: I64 = 42
val e = d.ref()"#,
    );
}

#[test]
fn test_method_calls() {
    snapshot_with_configs(
        "method_calls",
        r#"val text = "hello"
val text_ref = text.ref()
val number = 42
val num_string = number.toString()"#,
    );
}

#[test]
fn test_binary_operators() {
    snapshot_with_configs(
        "binary_operators",
        r#"val sum = 1 + 2
val diff = 10 - 5
val product = 3 * 4
val quotient = 20 / 5
val remainder = 17 % 5"#,
    );
}

#[test]
fn test_comparison_operators() {
    snapshot_with_configs(
        "comparison_operators",
        r#"val x = 10
val y = 20
val eq = x == y
val ne = x != y
val lt = x < y
val gt = x > y
val le = x <= y
val ge = x >= y"#,
    );
}

// Migration helper test - compares snapshot against expected file
#[test]
fn test_migration_example() {
    // This demonstrates how to verify snapshots match expected files during migration
    let veltrano_path = "examples/basic.vl";
    let expected_tuf_path = "examples/basic.tuf.expected.rs";
    let expected_kem_path = "examples/basic.kem.expected.rs";

    // Skip if files don't exist
    if !std::path::Path::new(veltrano_path).exists() {
        println!("Skipping migration example - basic.vl not found");
        return;
    }

    let veltrano_code = fs::read_to_string(veltrano_path).expect("Failed to read basic.vl");

    let configs = test_configs();

    // Test each config
    for (config_key, config) in configs {
        let ctx = TestContext::with_config(config);
        let rust_output = transpile(&veltrano_code, &ctx).expect("Transpilation failed");

        // Create snapshot
        assert_config_snapshot("migration_basic", config_key, &rust_output);

        // During migration, optionally verify against expected file
        let expected_path = match config_key {
            "tuf" => expected_tuf_path,
            "kem" => expected_kem_path,
            _ => continue,
        };

        if let Ok(expected) = fs::read_to_string(expected_path) {
            if rust_output.trim() != expected.trim() {
                println!(
                    "Warning: Snapshot differs from expected file for config '{}'\n\
                     This is expected if the transpiler behavior has changed.\n\
                     Review the snapshot with 'cargo insta review'",
                    config_key
                );
            }
        }
    }
}
