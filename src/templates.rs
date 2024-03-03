use crate::manifest::{BuiltinProfile, TomlProfileTemplate};
use crate::toml::{TemplateEntry, TomlTableTemplate};

/// Template that focuses on quick compile time.
pub fn fast_compile_template() -> TomlProfileTemplate {
    TomlProfileTemplate {
        inherits: BuiltinProfile::Dev,
        template: TomlTableTemplate {
            fields: vec![TemplateEntry::int("debug", 0)],
        },
    }
}

/// Template that focuses on maximum runtime performance.
pub fn fast_runtime_template() -> TomlProfileTemplate {
    TomlProfileTemplate {
        inherits: BuiltinProfile::Release,
        template: TomlTableTemplate {
            fields: vec![
                TemplateEntry::bool("lto", true),
                TemplateEntry::int("codegen-units", 1),
                TemplateEntry::string("panic", "abort"),
            ],
        },
    }
}

/// Template that template focuses on minimal binary size.
pub fn min_size_template() -> TomlProfileTemplate {
    TomlProfileTemplate {
        inherits: BuiltinProfile::Release,
        template: TomlTableTemplate {
            fields: vec![
                TemplateEntry::int("debug", 0),
                TemplateEntry::bool("strip", true),
                TemplateEntry::string("opt-level", "z"),
                TemplateEntry::bool("lto", true),
                TemplateEntry::int("codegen-units", 1),
                TemplateEntry::string("panic", "abort"),
            ],
        },
    }
}
