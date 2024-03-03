use std::string::ToString;

use crate::toml::{BuiltinProfile, TomlProfileTemplate, TomlTableTemplate, TomlValue};

pub fn fast_compile_template() -> TomlProfileTemplate {
    TomlProfileTemplate {
        inherits: BuiltinProfile::Dev,
        template: TomlTableTemplate {
            fields: vec![("debug".to_string(), TomlValue::Int(0))],
        },
    }
}
