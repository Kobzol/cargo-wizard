use indexmap::IndexMap;

use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;

/// A set of Cargo profile items and .cargo/config.toml config items that can be applied to a
/// Cargo workspace.
#[derive(Debug)]
pub struct Template {
    pub profile: ProfileTemplate,
    // pub conf: ProfileTemplate,
}

#[derive(Debug)]
pub struct ProfileTemplate {
    pub inherits: BuiltinProfile,
    pub items: IndexMap<ProfileItemId, TomlValue>,
}

pub struct TemplateBuilder {
    inherits: BuiltinProfile,
    profile: IndexMap<ProfileItemId, TomlValue>,
    // config: IndexMap<ConfigItemId, TomlValue>,
}

impl TemplateBuilder {
    pub fn new(inherits: BuiltinProfile) -> Self {
        Self {
            inherits,
            profile: Default::default(),
        }
    }

    pub fn profile_item(mut self, id: ProfileItemId, value: TomlValue) -> Self {
        assert!(self.profile.insert(id, value).is_none());
        self
    }

    pub fn build(self) -> Template {
        let TemplateBuilder { inherits, profile } = self;
        Template {
            profile: ProfileTemplate {
                inherits,
                items: profile,
            },
        }
    }
}

/// Identifier of a specific item of the profile part of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ProfileItemId {
    DebugInfo,
    Strip,
    Lto,
}

/// Identifier of a specific item of the config part of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ConfigItemId {
    TargetCpu,
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
pub struct CargoOption<Id> {
    id: Id,
    possible_values: Vec<PossibleValue>,
}

impl<Id: Copy> CargoOption<Id> {
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn possible_values(&self) -> &[PossibleValue] {
        &self.possible_values
    }
}

/// Known options from Cargo, containing descriptions and possible values.
pub struct KnownCargoOptions {
    profile: Vec<CargoOption<ProfileItemId>>,
    config: Vec<CargoOption<ConfigItemId>>,
}

impl KnownCargoOptions {
    pub fn new() -> Self {
        Self {
            profile: vec![
                CargoOption {
                    id: ProfileItemId::DebugInfo,
                    possible_values: vec![
                        PossibleValue::new("Disable debuginfo", TomlValue::Bool(false)),
                        PossibleValue::new("Enable debuginfo", TomlValue::Bool(true)),
                    ],
                },
                // CargoOption {
                //     id: ConfigItemId::Strip,
                //     possible_values: vec![
                //         PossibleValue::new("Do not strip anything", TomlValue::Bool(false)),
                //         PossibleValue::new(
                //             "Strip debug info",
                //             TomlValue::String("debuginfo".to_string()),
                //         ),
                //         PossibleValue::new(
                //             "Strip symbols",
                //             TomlValue::String("symbols".to_string()),
                //         ),
                //         PossibleValue::new("Strip debug info and symbols", TomlValue::Bool(true)),
                //     ],
                // },
            ],
            config: vec![CargoOption {
                id: ConfigItemId::TargetCpu,
                possible_values: vec![PossibleValue::new(
                    "Target CPU",
                    TomlValue::String("native".to_string()),
                )],
            }],
        }
    }

    pub fn get_profile(&self) -> &[CargoOption<ProfileItemId>] {
        &self.profile
    }

    pub fn get_config(&self) -> &[CargoOption<ConfigItemId>] {
        &self.config
    }
}
