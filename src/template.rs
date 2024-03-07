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
    CodegenUnits,
    Panic,
    OptimizationLevel,
}

/// Identifier of a specific item of the config part of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ConfigItemId {
    TargetCpu,
}
