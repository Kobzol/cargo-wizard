use toml_edit::Formatted;

#[derive(Clone, Debug)]
pub struct TableItem {
    pub name: String,
    pub value: TomlValue,
}

impl TableItem {
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
#[derive(Clone)]
pub struct TomlTableTemplate {
    pub items: Vec<TableItem>,
}

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
