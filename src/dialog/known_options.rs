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
}

impl TemplateItemMedata {
    fn new(values: &[PossibleValue]) -> Self {
        Self {
            values: values.to_vec(),
            custom_value: None,
        }
    }

    fn new_with_custom(values: &[PossibleValue], custom_value: TomlValueKind) -> Self {
        Self {
            values: values.to_vec(),
            custom_value: Some(custom_value),
        }
    }

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
}

impl KnownCargoOptions {
    pub fn get_all_ids() -> Vec<TemplateItemId> {
        vec![
            TemplateItemId::OptimizationLevel,
            TemplateItemId::Lto,
            TemplateItemId::CodegenUnits,
            TemplateItemId::TargetCpuInstructionSet,
            TemplateItemId::Panic,
            TemplateItemId::DebugInfo,
            TemplateItemId::Strip,
        ]
    }

    pub fn get_metadata(id: TemplateItemId) -> TemplateItemMedata {
        match id {
            TemplateItemId::OptimizationLevel => TemplateItemMedata::new(&[
                PossibleValue::new("No optimizations", TomlValue::Int(0)),
                PossibleValue::new("Basic optimizations", TomlValue::Int(1)),
                PossibleValue::new("Some optimizations", TomlValue::Int(2)),
                PossibleValue::new("All optimizations", TomlValue::Int(3)),
                PossibleValue::new(
                    "Optimize for small size",
                    TomlValue::String("s".to_string()),
                ),
                PossibleValue::new(
                    "Optimize for even smaller size",
                    TomlValue::String("z".to_string()),
                ),
            ]),
            TemplateItemId::Lto => TemplateItemMedata::new(&[
                PossibleValue::new("Disable LTO", TomlValue::String("off".to_string())),
                PossibleValue::new("Thin local LTO (default)", TomlValue::Bool(false)),
                PossibleValue::new("Thin LTO", TomlValue::String("thin".to_string())),
                PossibleValue::new("Fat LTO", TomlValue::Bool(true)),
            ]),
            TemplateItemId::CodegenUnits => TemplateItemMedata::new_with_custom(
                &[PossibleValue::new("1 CGU", TomlValue::Int(1))],
                TomlValueKind::Int,
            ),
            TemplateItemId::Panic => TemplateItemMedata::new(&[
                PossibleValue::new("Unwind", TomlValue::String("unwind".to_string())),
                PossibleValue::new("Abort", TomlValue::String("abort".to_string())),
            ]),
            TemplateItemId::DebugInfo => TemplateItemMedata::new(&[
                PossibleValue::new("Disable debuginfo", TomlValue::Bool(false)),
                PossibleValue::new(
                    "Enable line directives",
                    TomlValue::String("line-directives-only".to_string()),
                ),
                PossibleValue::new(
                    "Enable line tables",
                    TomlValue::String("line-tables-only".to_string()),
                ),
                PossibleValue::new("Limited debuginfo", TomlValue::Int(1)),
                PossibleValue::new("Full debuginfo", TomlValue::Bool(true)),
            ]),
            TemplateItemId::Strip => TemplateItemMedata::new(&[
                PossibleValue::new("Do not strip anything", TomlValue::Bool(false)),
                PossibleValue::new(
                    "Strip debug info",
                    TomlValue::String("debuginfo".to_string()),
                ),
                PossibleValue::new("Strip symbols", TomlValue::String("symbols".to_string())),
                PossibleValue::new("Strip debug info and symbols", TomlValue::Bool(true)),
            ]),
            TemplateItemId::TargetCpuInstructionSet => TemplateItemMedata::new_with_custom(
                &[PossibleValue::new(
                    "Native (best for the local CPU)",
                    TomlValue::string("native"),
                )],
                TomlValueKind::String,
            ),
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
