pub struct CliConfig {
    use_colors: bool,
}

impl CliConfig {
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    pub fn colors_enabled(&self) -> bool {
        self.use_colors
    }
}
