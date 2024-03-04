use crate::toml::{TableItem, TomlTableTemplate};
use crate::workspace::config::ConfigTemplate;
use crate::workspace::manifest::BuiltinProfile;
use crate::TomlProfileTemplate;

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

pub struct Template {
    pub profile: TomlProfileTemplate,
    pub config: Option<ConfigTemplate>,
}

/// Template that focuses on quick compile time.
pub fn fast_compile_template() -> Template {
    Template {
        profile: TomlProfileTemplate {
            inherits: BuiltinProfile::Dev,
            template: TomlTableTemplate {
                items: vec![TableItem::int("debug", 0)],
            },
        },
        config: None,
    }
}

/// Template that focuses on maximum runtime performance.
pub fn fast_runtime_template() -> Template {
    Template {
        profile: TomlProfileTemplate {
            inherits: BuiltinProfile::Release,
            template: TomlTableTemplate {
                items: vec![
                    TableItem::bool("lto", true),
                    TableItem::int("codegen-units", 1),
                    TableItem::string("panic", "abort"),
                ],
            },
        },
        config: Some(ConfigTemplate {
            flags: vec!["-Ctarget-cpu=native".to_string()],
        }),
    }
}

/// Template that template focuses on minimal binary size.
pub fn min_size_template() -> Template {
    Template {
        profile: TomlProfileTemplate {
            inherits: BuiltinProfile::Release,
            template: TomlTableTemplate {
                items: vec![
                    TableItem::int("debug", 0),
                    TableItem::bool("strip", true),
                    TableItem::string("opt-level", "z"),
                    TableItem::bool("lto", true),
                    TableItem::int("codegen-units", 1),
                    TableItem::string("panic", "abort"),
                ],
            },
        },
        config: None,
    }
}
