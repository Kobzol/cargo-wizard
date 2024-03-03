use toml_edit::Formatted;

/// A template that contains prefilled values that can add or replace fields in a TOML table.
pub struct TomlTableTemplate {
    pub fields: Vec<(String, TomlValue)>,
}

pub enum BuiltinProfile {
    Dev,
    Release,
}

pub struct TomlProfileTemplate {
    pub inherits: BuiltinProfile,
    pub template: TomlTableTemplate,
}

pub enum TomlValue {
    Int(i64),
    Bool(bool),
    String(String),
}

impl TomlValue {
    pub fn to_toml_value(&self) -> toml_edit::Value {
        match self {
            TomlValue::Int(value) => toml_edit::Value::Integer(Formatted::new(*value)),
            TomlValue::Bool(value) => toml_edit::Value::Boolean(Formatted::new(*value)),
            TomlValue::String(value) => toml_edit::Value::String(Formatted::new(value.clone())),
        }
    }
}
