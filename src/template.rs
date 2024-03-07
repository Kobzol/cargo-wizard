use indexmap::IndexMap;

use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;

/// A set of Cargo profile items and .cargo/config.toml config items that can be applied to a
/// Cargo workspace.
#[derive(Debug)]
pub struct Template {
    pub profile: ProfileTemplate,
    pub config: ConfigTemplate,
}

#[derive(Debug)]
pub struct ProfileTemplate {
    pub inherits: BuiltinProfile,
    pub items: IndexMap<ProfileItemId, TomlValue>,
}

#[derive(Debug)]
pub struct ConfigTemplate {
    pub items: IndexMap<ConfigItemId, String>,
}

pub struct TemplateBuilder {
    inherits: BuiltinProfile,
    profile: IndexMap<ProfileItemId, TomlValue>,
    config: IndexMap<ConfigItemId, String>,
}

impl TemplateBuilder {
    pub fn new(inherits: BuiltinProfile) -> Self {
        Self {
            inherits,
            profile: Default::default(),
            config: Default::default(),
        }
    }

    pub fn profile_item(mut self, id: ProfileItemId, value: TomlValue) -> Self {
        assert!(self.profile.insert(id, value).is_none());
        self
    }

    pub fn config_item(mut self, id: ConfigItemId, value: String) -> Self {
        assert!(self.config.insert(id, value).is_none());
        self
    }

    pub fn build(self) -> Template {
        let TemplateBuilder {
            inherits,
            profile,
            config,
        } = self;
        Template {
            profile: ProfileTemplate {
                inherits,
                items: profile,
            },
            config: ConfigTemplate { items: config },
        }
    }
}

/// Identifier of a specific item of the profile part of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ProfileItemId {
    DebugInfo,
    Strip,
    Lto,
    CodegenUnits,
    Panic,
    OptimizationLevel,
}

/// Identifier of a specific item of the config part of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ConfigItemId {
    TargetCpu,
}
