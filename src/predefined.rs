use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;
use crate::{Template, TemplateBuilder, TemplateItemId};

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
        .item(TemplateItemId::ProfileDebugInfo, TomlValue::Int(0))
        .build()
}

/// Template that focuses on maximum runtime performance.
pub fn fast_runtime_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Release)
        .item(TemplateItemId::ProfileLto, TomlValue::Bool(true))
        .build()
    //TableItem::int("codegen-units", 1),
    //                     TableItem::string("panic", "abort"),
    // config: Some(ConfigTemplate {
    //     rustflags: vec!["-Ctarget-cpu=native".to_string()],
    // }),
}

/// Template that template focuses on minimal binary size.
pub fn min_size_template() -> Template {
    TemplateBuilder::new(BuiltinProfile::Release)
        .item(TemplateItemId::ProfileLto, TomlValue::Bool(true))
        .build()
    // items: vec![
    //     TableItem::int("debug", 0),
    //     TableItem::bool("strip", true),
    //     TableItem::string("opt-level", "z"),
    //     TableItem::bool("lto", true),
    //     TableItem::int("codegen-units", 1),
    //     TableItem::string("panic", "abort"),
    // ],
}
