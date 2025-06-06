use std::collections::HashMap;
use veltrano::config::Config;

/// Returns a HashMap of predefined config instances for testing.
/// Keys are used in expected output filenames (e.g., "example.tuf.expected.rs").
pub fn test_configs() -> HashMap<&'static str, Config> {
    let mut configs = HashMap::new();
    configs.insert(
        "tuf",
        Config {
            preserve_comments: false,
        },
    );
    configs.insert(
        "kem",
        Config {
            preserve_comments: true,
        },
    );
    configs
}
