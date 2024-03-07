use indexmap::IndexMap;

use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;

/// A set of Cargo profile items and .cargo/config.toml config items that can be applied to a
/// Cargo workspace.
#[derive(Debug)]
pub struct Template {
    pub inherits: BuiltinProfile,
    pub items: IndexMap<TemplateItemId, TomlValue>,
}

pub struct TemplateBuilder {
    inherits: BuiltinProfile,
    profile: IndexMap<TemplateItemId, TomlValue>,
}

impl TemplateBuilder {
    pub fn new(inherits: BuiltinProfile) -> Self {
        Self {
            inherits,
            profile: Default::default(),
        }
    }

    pub fn item(mut self, id: TemplateItemId, value: TomlValue) -> Self {
        assert!(self.profile.insert(id, value).is_none());
        self
    }

    pub fn build(self) -> Template {
        let TemplateBuilder { inherits, profile } = self;
        Template {
            inherits,
            items: profile,
        }
    }
}

/// Identifier of a specific item of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum TemplateItemId {
    DebugInfo,
    Strip,
    Lto,
    CodegenUnits,
    Panic,
    OptimizationLevel,
    TargetCpuInstructionSet,
}
