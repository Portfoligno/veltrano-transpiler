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
    /// Keys are used in expected output filenames (e.g., "example.default.expected.rs").
    #[cfg(test)]
    pub fn test_configs() -> std::collections::HashMap<&'static str, Config> {
        let mut configs = std::collections::HashMap::new();
        configs.insert("default", Config { preserve_comments: false });
        configs.insert("comments", Config { preserve_comments: true });
        configs
    }
}
