//! Example test demonstrating snapshot testing

mod common;
mod test_configs;

use common::snapshot_utils::{assert_error_snapshot, assert_transpiler_snapshot};
use common::{transpile, TestContext};
use veltrano::config::Config;

#[test]
fn test_basic_val_declaration_snapshot() {
    let veltrano_code = "val x = 42";
    let ctx = TestContext::with_config(Config {
        preserve_comments: false,
    });

    let rust_output = transpile(veltrano_code, &ctx).expect("Transpilation should succeed");

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
    use test_configs::test_configs;

    let veltrano_code = "val x = 42 // important";
    let configs = test_configs();

    for (config_key, config) in configs {
        let ctx = TestContext::with_config(config);
        let output = transpile(veltrano_code, &ctx).expect("Transpilation should succeed");
        assert_config_snapshot("val_with_comment", config_key, &output);
    }
}
