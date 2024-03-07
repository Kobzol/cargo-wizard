use cargo_wizard::{TemplateItemId, TomlValue};

/// Known options from Cargo, containing descriptions and possible values.
pub struct KnownCargoOptions;

#[derive(Copy, Clone)]
pub enum TomlValueKind {
    Int,
    String,
}

impl TomlValueKind {
    fn matches_value(&self, value: &TomlValue) -> bool {
        match self {
            TomlValueKind::Int if matches!(value, TomlValue::Int(_)) => true,
            TomlValueKind::String if matches!(value, TomlValue::String(_)) => true,
            TomlValueKind::Int | TomlValueKind::String => false,
        }
    }
}

pub enum SelectedPossibleValue {
    Constant { index: usize, value: TomlValue },
    Custom { value: TomlValue },
    None,
}

pub struct TemplateItemMedata {
    values: Vec<PossibleValue>,
    custom_value: Option<TomlValueKind>,
    requires_nightly: bool,
}

impl TemplateItemMedata {
    pub fn get_selected_value(&self, value: TomlValue) -> SelectedPossibleValue {
        if let Some(index) = self.values.iter().position(|v| v.value == value) {
            return SelectedPossibleValue::Constant { value, index };
        } else if let Some(custom) = &self.custom_value {
            if custom.matches_value(&value) {
                return SelectedPossibleValue::Custom { value };
            }
        }
        SelectedPossibleValue::None
    }

    pub fn get_possible_values(&self) -> &[PossibleValue] {
        &self.values
    }

    pub fn get_custom_value_kind(&self) -> Option<TomlValueKind> {
        self.custom_value
    }

    pub fn requires_nightly(&self) -> bool {
        self.requires_nightly
    }
}

#[derive(Default)]
struct MetadataBuilder {
    values: Vec<PossibleValue>,
    custom_value: Option<TomlValueKind>,
    requires_nightly: bool,
}

impl MetadataBuilder {
    fn build(self) -> TemplateItemMedata {
        let MetadataBuilder {
            values,
            custom_value,
            requires_nightly,
        } = self;
        TemplateItemMedata {
            values,
            custom_value,
            requires_nightly,
        }
    }

    fn value(mut self, description: &'static str, value: TomlValue) -> Self {
        self.values.push(PossibleValue::new(description, value));
        self
    }

    fn int(self, description: &'static str, value: i64) -> Self {
        self.value(description, TomlValue::Int(value))
    }

    fn bool(self, description: &'static str, value: bool) -> Self {
        self.value(description, TomlValue::Bool(value))
    }

    fn string(self, description: &'static str, value: &str) -> Self {
        self.value(description, TomlValue::String(value.to_string()))
    }

    fn custom_value(mut self, kind: TomlValueKind) -> Self {
        self.custom_value = Some(kind);
        self
    }

    fn requires_nightly(mut self) -> Self {
        self.requires_nightly = true;
        self
    }
}

impl KnownCargoOptions {
    pub fn get_all_ids() -> Vec<TemplateItemId> {
        vec![
            TemplateItemId::OptimizationLevel,
            TemplateItemId::Lto,
            TemplateItemId::CodegenUnits,
            TemplateItemId::TargetCpuInstructionSet,
            TemplateItemId::CodegenBackend,
            TemplateItemId::Panic,
            TemplateItemId::DebugInfo,
            TemplateItemId::Strip,
        ]
    }

    pub fn get_metadata(id: TemplateItemId) -> TemplateItemMedata {
        match id {
            TemplateItemId::OptimizationLevel => MetadataBuilder::default()
                .int("No optimizations", 0)
                .int("Basic optimizations", 1)
                .int("Some optimizations", 2)
                .int("All optimizations", 3)
                .string("Optimize for small size", "s")
                .string("Optimize for even smaller size", "z")
                .build(),
            TemplateItemId::Lto => MetadataBuilder::default()
                .string("Disable LTO", "off")
                .bool("Thin local LTO", false)
                .string("Thin LTO", "thin")
                .bool("Fat LTO", true)
                .build(),
            TemplateItemId::CodegenUnits => MetadataBuilder::default()
                .int("1 CGU", 1)
                .custom_value(TomlValueKind::Int)
                .build(),
            TemplateItemId::Panic => MetadataBuilder::default()
                .string("Unwind", "unwind")
                .string("Abort", "abort")
                .build(),
            TemplateItemId::DebugInfo => MetadataBuilder::default()
                .bool("Disable debuginfo", false)
                .string("Enable line directives", "line-directives-only")
                .string("Enable line tables", "line-tables-only")
                .int("Limited debuginfo", 1)
                .bool("Full debuginfo", true)
                .build(),
            TemplateItemId::Strip => MetadataBuilder::default()
                .bool("Do not strip anything", false)
                .string("Strip debug info", "debuginfo")
                .string("Strip symbols", "symbols")
                .bool("Strip debug info and symbols", true)
                .build(),
            TemplateItemId::TargetCpuInstructionSet => MetadataBuilder::default()
                .string("Native (best for the local CPU)", "native")
                .custom_value(TomlValueKind::String)
                .build(),
            TemplateItemId::CodegenBackend => MetadataBuilder::default()
                .string("LLVM", "llvm")
                .string("Cranelift", "cranelift")
                .requires_nightly()
                .build(),
        }
    }
}

/// Possible value of a Cargo profile or a Cargo config, along with a description of what it does.
#[derive(Debug, Clone)]
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

/// Test that the predefined templates can be created without panicking.
#[cfg(test)]
mod tests {
    use crate::dialog::known_options::KnownCargoOptions;

    #[test]
    fn get_profile_id_possible_values() {
        for id in KnownCargoOptions::get_all_ids() {
            assert!(!KnownCargoOptions::get_metadata(id)
                .get_possible_values()
                .is_empty());
        }
    }
}
