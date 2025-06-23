//! Utilities for snapshot testing with insta

use insta::{assert_snapshot, Settings};

/// Configure insta settings for consistent snapshots
pub fn with_settings<F>(f: F)
where
    F: FnOnce(),
{
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path("../snapshots");
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(f);
}

/// Helper for creating transpiler output snapshots
pub fn assert_transpiler_snapshot(name: &str, veltrano_code: &str, rust_output: &str) {
    with_settings(|| {
        assert_snapshot!(
            name,
            format!(
                "=== INPUT (Veltrano) ===\n{}\n\n=== OUTPUT (Rust) ===\n{}",
                veltrano_code, rust_output
            )
        );
    });
}

/// Helper for creating error snapshots with consistent formatting
pub fn assert_error_snapshot(name: &str, error: &dyn std::fmt::Display) {
    with_settings(|| {
        assert_snapshot!(name, format!("{:#}", error));
    });
}

/// Helper for config-specific snapshots
pub fn assert_config_snapshot(base_name: &str, config_name: &str, content: &str) {
    let snapshot_name = format!("{}.{}", base_name, config_name);
    with_settings(|| {
        assert_snapshot!(snapshot_name, content);
    });
}

/// Create a snapshot with source location info
pub fn assert_snapshot_with_source(name: &str, content: &str, source_file: &str, line: u32) {
    with_settings(|| {
        assert_snapshot!(
            name,
            format!("Source: {}:{}\n---\n{}", source_file, line, content)
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_utils_compile() {
        // Just ensure the module compiles
        with_settings(|| {
            // Settings are properly configured
        });
    }
}
