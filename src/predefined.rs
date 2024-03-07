use crate::template::{TemplateBuilder, TemplateItemId};
use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;
use crate::Template;

/// Enumeration of predefined templates.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PredefinedTemplateKind {
    /// Profile designed for fast compilation times.
    FastCompile,
    /// Profile designed for fast runtime performance.
    FastRuntime,
    /// Profile designed for minimal binary size.
    MinSize,
}

impl PredefinedTemplateKind {
    pub fn build_template(&self) -> Template {
        match self {
            PredefinedTemplateKind::FastCompile => fast_compile_template(),
            PredefinedTemplateKind::FastRuntime => fast_runtime_template(),
            PredefinedTemplateKind::MinSize => min_size_template(),
        }
    }
}

/// Template that focuses on quick compile time.
pub fn fast_compile_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Dev)
        .item(TemplateItemId::DebugInfo, TomlValue::int(0))
        .build()
}

/// Template that focuses on maximum runtime performance.
pub fn fast_runtime_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Release)
        .item(TemplateItemId::Lto, TomlValue::bool(true))
        .item(TemplateItemId::CodegenUnits, TomlValue::int(1))
        .item(TemplateItemId::Panic, TomlValue::string("abort"))
        .item(
            TemplateItemId::TargetCpuInstructionSet,
            TomlValue::string("native"),
        )
        .build()
}

/// Template that template focuses on minimal binary size.
pub fn min_size_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Release)
        .item(TemplateItemId::DebugInfo, TomlValue::int(0))
        .item(TemplateItemId::Strip, TomlValue::bool(true))
        .item(TemplateItemId::Lto, TomlValue::bool(true))
        .item(TemplateItemId::OptimizationLevel, TomlValue::string("z"))
        .item(TemplateItemId::CodegenUnits, TomlValue::int(1))
        .item(TemplateItemId::Panic, TomlValue::string("abort"))
        .build()
}

/// Test that the predefined templates can be created without panicking.
#[cfg(test)]
mod tests {
    use crate::{fast_compile_template, fast_runtime_template, min_size_template};

    #[test]
    fn create_fast_compile_template() {
        fast_compile_template();
    }

    #[test]
    fn create_fast_runtime_template() {
        fast_runtime_template();
    }

    #[test]
    fn create_min_size_template() {
        min_size_template();
    }
}
