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

