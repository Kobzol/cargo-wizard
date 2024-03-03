use cargo_wizard::{
    fast_compile_template, fast_runtime_template, min_size_template, TomlProfileTemplate,
};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PredefinedTemplate {
    /// Profile designed for fast compilation times.
    FastCompile,
    /// Profile designed for fast runtime performance.
    FastRuntime,
    /// Profile designed for minimal binary size.
    MinSize,
}

impl PredefinedTemplate {
    pub fn resolve_to_template(&self) -> TomlProfileTemplate {
        match self {
            PredefinedTemplate::FastCompile => fast_compile_template(),
            PredefinedTemplate::FastRuntime => fast_runtime_template(),
            PredefinedTemplate::MinSize => min_size_template(),
        }
    }
}
