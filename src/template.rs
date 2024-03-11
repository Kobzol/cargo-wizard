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
        self.profile.insert(id, value);
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

/// Default properties of the dev profile.
pub fn dev_profile() -> TemplateBuilder {
    TemplateBuilder::new(BuiltinProfile::Dev)
        .item(TemplateItemId::OptimizationLevel, TomlValue::Int(0))
        .item(TemplateItemId::DebugInfo, TomlValue::Bool(true))
        .item(TemplateItemId::Strip, TomlValue::String("none".to_string()))
        .item(TemplateItemId::Lto, TomlValue::Bool(false))
        .item(TemplateItemId::CodegenUnits, TomlValue::Int(256))
        .item(TemplateItemId::Incremental, TomlValue::Bool(true))
}

/// Default properties of the release profile.
pub fn release_profile() -> TemplateBuilder {
    TemplateBuilder::new(BuiltinProfile::Release)
        .item(TemplateItemId::OptimizationLevel, TomlValue::Int(3))
        .item(TemplateItemId::DebugInfo, TomlValue::Bool(false))
        .item(TemplateItemId::Strip, TomlValue::String("none".to_string()))
        .item(TemplateItemId::Lto, TomlValue::Bool(false))
        .item(TemplateItemId::CodegenUnits, TomlValue::Int(16))
        .item(TemplateItemId::Incremental, TomlValue::Bool(false))
}

/// Identifier of a specific item of a template.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum TemplateItemId {
    // Do not forget to modify CargoKnownOptions when adding new variants to this enum
    DebugInfo,
    SplitDebugInfo,
    Strip,
    Lto,
    CodegenUnits,
    Panic,
    OptimizationLevel,
    Incremental,
    CodegenBackend,
    FrontendThreads,
    TargetCpuInstructionSet,
    Linker,
}

/// Describes options for applying templates
#[derive(Debug, Default)]
pub struct WizardOptions {
    /// Include template items that require a nightly compiler.
    nightly_items: bool,
}

impl WizardOptions {
    pub fn nightly_items_enabled(&self) -> bool {
        self.nightly_items
    }

    pub fn with_nightly_items(mut self) -> Self {
        self.nightly_items = true;
        self
    }
}
