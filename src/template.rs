use std::collections::HashMap;

use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;

/// A set of Cargo profile items and .cargo/config.toml config items that can be applied to a
/// Cargo workspace.
#[derive(Debug, Clone)]
pub struct Template {
    pub inherits: BuiltinProfile,
    pub items: HashMap<TemplateItemId, TomlValue>,
}

pub struct TemplateBuilder {
    inherits: BuiltinProfile,
    items: HashMap<TemplateItemId, TomlValue>,
}

impl TemplateBuilder {
    pub fn new(inherits: BuiltinProfile) -> Self {
        Self {
            inherits,
            items: Default::default(),
        }
    }

    pub fn item(mut self, id: TemplateItemId, value: TomlValue) -> Self {
        self.items.insert(id, value);
        self
    }

    pub fn build(self) -> Template {
        let TemplateBuilder { inherits, items } = self;
        Template { inherits, items }
    }
}

/// Identifier of a specific item of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum TemplateItemId {
    ProfileDebugInfo,
    ProfileStrip,
    ProfileLto,
}

/// Possible value of a Cargo profile or a Cargo config, along with a description of what it does.
#[derive(Debug)]
pub struct PossibleValue {
    description: String,
    value: TomlValue,
}

impl PossibleValue {
    fn new(description: &'static str, value: TomlValue) -> Self {
        Self {
            value,
            description: description.to_string(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn value(&self) -> &TomlValue {
        &self.value
    }
}

#[derive(Debug)]
pub struct CargoOption {
    id: TemplateItemId,
    possible_values: Vec<PossibleValue>,
}

impl CargoOption {
    pub fn id(&self) -> TemplateItemId {
        self.id
    }

    pub fn possible_values(&self) -> &[PossibleValue] {
        &self.possible_values
    }
}

/// Known options from Cargo, containing descriptions and possible values.
pub struct KnownCargoOptions {
    options: Vec<CargoOption>,
}

impl KnownCargoOptions {
    pub fn new() -> Self {
        Self {
            options: vec![
                CargoOption {
                    id: TemplateItemId::ProfileDebugInfo,
                    possible_values: vec![
                        PossibleValue::new("Disable debuginfo", TomlValue::Bool(false)),
                        PossibleValue::new("Enable debuginfo", TomlValue::Bool(true)),
                    ],
                },
                CargoOption {
                    id: TemplateItemId::ProfileStrip,
                    possible_values: vec![
                        PossibleValue::new("Do not strip anything", TomlValue::Bool(false)),
                        PossibleValue::new(
                            "Strip debug info",
                            TomlValue::String("debuginfo".to_string()),
                        ),
                        PossibleValue::new(
                            "Strip symbols",
                            TomlValue::String("symbols".to_string()),
                        ),
                        PossibleValue::new("Strip debug info and symbols", TomlValue::Bool(true)),
                    ],
                },
            ],
        }
    }

    pub fn get_options(&self) -> &[CargoOption] {
        &self.options
    }
}
