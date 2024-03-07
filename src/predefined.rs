use crate::template::ProfileItemId;
use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;
use crate::{ConfigItemId, Template, TemplateBuilder};

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
        .profile_item(ProfileItemId::DebugInfo, TomlValue::Int(0))
        .build()
}

/// Template that focuses on maximum runtime performance.
pub fn fast_runtime_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Release)
        .profile_item(ProfileItemId::Lto, TomlValue::Bool(true))
        .profile_item(ProfileItemId::CodegenUnits, TomlValue::Int(1))
        .profile_item(ProfileItemId::Panic, TomlValue::String("abort".to_string()))
        .config_item(ConfigItemId::TargetCpu, "native".to_string())
        .build()
}

/// Template that template focuses on minimal binary size.
pub fn min_size_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Release)
        .profile_item(ProfileItemId::DebugInfo, TomlValue::Int(0))
        .profile_item(ProfileItemId::Strip, TomlValue::Bool(true))
        .profile_item(ProfileItemId::Lto, TomlValue::Bool(true))
        .profile_item(
            ProfileItemId::OptimizationLevel,
            TomlValue::String("z".to_string()),
        )
        .profile_item(ProfileItemId::CodegenUnits, TomlValue::Int(1))
        .profile_item(ProfileItemId::Panic, TomlValue::String("abort".to_string()))
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
