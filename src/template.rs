use indexmap::IndexMap;

use crate::toml::TomlValue;
use crate::workspace::manifest::BuiltinProfile;

/// A set of Cargo profile items and .cargo/config.toml config items that can be applied to a
/// Cargo workspace.
#[derive(Debug)]
pub struct Template {
    inherits: BuiltinProfile,
    items: IndexMap<TemplateItemId, TomlValue>,
}

impl Template {
    pub fn inherits(&self) -> BuiltinProfile {
        self.inherits
    }

    pub fn iter_items(&self) -> impl Iterator<Item = (TemplateItemId, &TomlValue)> {
        self.items.iter().map(|(id, value)| (*id, value))
    }

    pub fn get_item(&self, id: TemplateItemId) -> Option<&TomlValue> {
        self.items.get(&id)
    }

    pub fn insert_item(&mut self, id: TemplateItemId, value: TomlValue) {
        self.items.insert(id, value);
    }

    pub fn remove_item(&mut self, id: TemplateItemId) {
        self.items.shift_remove(&id);
    }
}

#[doc(hidden)]
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
    CodegenBackend,
}
