use toml_edit::Formatted;

/// Representation of a numeric, boolean or a string TOML value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TomlValue {
    Int(i64),
    Bool(bool),
    String(String),
}

impl TomlValue {
    pub fn int(value: i64) -> Self {
        TomlValue::Int(value)
    }

    pub fn bool(value: bool) -> Self {
        TomlValue::Bool(value)
    }

    pub fn string(value: &str) -> Self {
        TomlValue::String(value.to_string())
    }

    pub fn to_toml_value(&self) -> toml_edit::Value {
        match self {
            TomlValue::Int(value) => toml_edit::Value::Integer(Formatted::new(*value)),
            TomlValue::Bool(value) => toml_edit::Value::Boolean(Formatted::new(*value)),
            TomlValue::String(value) => toml_edit::Value::String(Formatted::new(value.clone())),
        }
    }
}
