use toml_edit::Formatted;

#[derive(Clone, Debug)]
pub struct TemplateEntry {
    pub name: String,
    pub value: TomlValue,
}

impl TemplateEntry {
    pub fn int(name: &str, value: i64) -> Self {
        Self {
            name: name.to_string(),
            value: TomlValue::Int(value),
        }
    }

    pub fn bool(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: TomlValue::Bool(value),
        }
    }

    pub fn string(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: TomlValue::String(value.to_string()),
        }
    }
}

/// A template that contains prefilled values that can add or replace fields in a TOML table.
pub struct TomlTableTemplate {
    pub fields: Vec<TemplateEntry>,
}

#[derive(Clone, Debug)]
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
