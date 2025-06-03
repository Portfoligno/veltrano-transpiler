#[derive(Debug, Clone)]
pub struct Config {
    pub preserve_comments: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preserve_comments: false,
        }
    }
}

impl Config {
    /// Returns a HashMap of predefined config instances for testing.
    /// Keys are used in expected output filenames (e.g., "example.tuf.expected.rs").
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn test_configs() -> std::collections::HashMap<&'static str, Config> {
        let mut configs = std::collections::HashMap::new();
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
}
