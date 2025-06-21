//! Example test demonstrating snapshot testing

mod common;

use common::{transpile, TestContext};
use common::snapshot_utils::{assert_transpiler_snapshot, assert_error_snapshot};
use veltrano::config::Config;

#[test]
fn test_basic_val_declaration_snapshot() {
    let veltrano_code = "val x = 42";
    let ctx = TestContext::with_config(Config {
        preserve_comments: false,
    });
    
    let rust_output = transpile(veltrano_code, &ctx)
        .expect("Transpilation should succeed");
    
    assert_transpiler_snapshot("basic_val_declaration", veltrano_code, &rust_output);
}

#[test]
fn test_parse_error_snapshot() {
    let invalid_code = "val x =";
    let ctx = TestContext::with_config(Config {
        preserve_comments: false,
    });
    
    match transpile(invalid_code, &ctx) {
        Ok(_) => panic!("Expected parse error"),
        Err(error) => {
            assert_error_snapshot("parse_error_missing_value", &error);
        }
    }
}

#[test]
fn test_config_specific_snapshot() {
    use common::snapshot_utils::assert_config_snapshot;
    
    let veltrano_code = "val x = 42 // important";
    
    // Test with comments preserved
    let ctx_with = TestContext::with_config(Config {
        preserve_comments: true,
    });
    let output_with = transpile(veltrano_code, &ctx_with)
        .expect("Transpilation should succeed");
    assert_config_snapshot("val_with_comment", "preserve", &output_with);
    
    // Test without comments
    let ctx_without = TestContext::with_config(Config {
        preserve_comments: false,
    });
    let output_without = transpile(veltrano_code, &ctx_without)
        .expect("Transpilation should succeed");
    assert_config_snapshot("val_with_comment", "strip", &output_without);
}
