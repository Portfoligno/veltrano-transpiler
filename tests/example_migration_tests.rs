//! Example of migrating integration tests to snapshots

mod common;
mod test_configs;

use common::{transpile, TestContext};
use common::snapshot_utils::assert_config_snapshot;
use test_configs::test_configs;
use std::fs;
use std::path::Path;

/// Migrate a single example file to snapshots
fn migrate_example_to_snapshot(example_name: &str) {
    let vl_path = format!("examples/{}.vl", example_name);
    
    // Skip if source doesn't exist
    if !Path::new(&vl_path).exists() {
        println!("Skipping {} - source file not found", example_name);
        return;
    }
    
    let veltrano_code = fs::read_to_string(&vl_path)
        .expect("Failed to read source file");
    
    let configs = test_configs();
    
    for (config_key, config) in configs {
        let ctx = TestContext::with_config(config);
        
        match transpile(&veltrano_code, &ctx) {
            Ok(rust_output) => {
                // Create snapshot
                assert_config_snapshot(
                    &format!("example_{}", example_name),
                    config_key,
                    &rust_output
                );
                
                // Optional: Compare with expected file during migration
                let expected_path = format!("examples/{}.{}.expected.rs", example_name, config_key);
                if let Ok(expected) = fs::read_to_string(&expected_path) {
                    if rust_output.trim() != expected.trim() {
                        println!(
                            "Note: {} differs from {}.{}.expected.rs - review snapshot",
                            example_name, example_name, config_key
                        );
                    }
                }
            }
            Err(e) => {
                // Some examples might be fail tests
                if !example_name.contains("fail") {
                    panic!("Unexpected transpilation failure for {}: {}", example_name, e);
                }
            }
        }
    }
}

#[test]
fn test_migrate_basic_function() {
    migrate_example_to_snapshot("basic_function");
}

#[test]
fn test_migrate_simple_types() {
    migrate_example_to_snapshot("simple_types");
}

#[test]
fn test_migrate_ownership_transfer() {
    migrate_example_to_snapshot("ownership_transfer");
}

#[test]
fn test_migrate_method_chaining() {
    migrate_example_to_snapshot("method_chaining");
}

#[test]
fn test_migrate_comments() {
    migrate_example_to_snapshot("comments");
}

// Batch migration test - migrate multiple examples at once
#[test]
#[ignore] // Run with --ignored to migrate all
fn test_migrate_all_examples() {
    let examples = vec![
        "basic_function",
        "simple_types",
        "ownership_transfer",
        "method_chaining",
        "comments",
        "control_flow",
        "pattern_matching",
        "type_inference",
    ];
    
    for example in examples {
        println!("Migrating {}...", example);
        migrate_example_to_snapshot(example);
    }
}
