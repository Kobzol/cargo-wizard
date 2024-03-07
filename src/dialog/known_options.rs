use cargo_wizard::{ProfileItemId, TomlValue};

/// Known options from Cargo, containing descriptions and possible values.
pub struct KnownCargoOptions;

impl KnownCargoOptions {
    pub fn get_profile_ids() -> Vec<ProfileItemId> {
        vec![
            ProfileItemId::DebugInfo,
            ProfileItemId::Strip,
            ProfileItemId::Lto,
            ProfileItemId::CodegenUnits,
            ProfileItemId::Panic,
            ProfileItemId::OptimizationLevel,
        ]
    }

    pub fn get_profile_possible_values(id: ProfileItemId) -> Vec<PossibleValue> {
        match id {
            ProfileItemId::DebugInfo => vec![
                PossibleValue::new("Disable debuginfo", TomlValue::Bool(false)),
                PossibleValue::new("Enable debuginfo", TomlValue::Bool(true)),
            ],
            ProfileItemId::Strip => vec![
                PossibleValue::new("Do not strip anything", TomlValue::Bool(false)),
                PossibleValue::new(
                    "Strip debug info",
                    TomlValue::String("debuginfo".to_string()),
                ),
                PossibleValue::new("Strip symbols", TomlValue::String("symbols".to_string())),
                PossibleValue::new("Strip debug info and symbols", TomlValue::Bool(true)),
            ],
            ProfileItemId::Lto => {
                vec![
                    PossibleValue::new("No LTO", TomlValue::Bool(false)),
                    PossibleValue::new("Thin LTO", TomlValue::String("thin".to_string())),
                    PossibleValue::new("Fat LTO", TomlValue::Bool(true)),
                ]
            }
            ProfileItemId::CodegenUnits => {
                vec![PossibleValue::new("1 CGU", TomlValue::Int(1))]
            }
            ProfileItemId::Panic => {
                vec![
                    PossibleValue::new("Unwind", TomlValue::String("unwind".to_string())),
                    PossibleValue::new("Abort", TomlValue::String("abort".to_string())),
                ]
            }
            ProfileItemId::OptimizationLevel => {
                vec![
                    PossibleValue::new("No optimizations", TomlValue::Int(0)),
                    PossibleValue::new("Some optimizations", TomlValue::Int(1)),
                    PossibleValue::new("Full optimizations", TomlValue::Int(2)),
                    PossibleValue::new(
                        "Optimize for small size",
                        TomlValue::String("z".to_string()),
                    ),
                ]
            }
        }
    }
    // config: vec![CargoOption {
    //     id: ConfigItemId::TargetCpu,
    //     possible_values: vec![PossibleValue::new(
    //         "Target CPU",
    //         TomlValue::String("native".to_string()),
    //     )],
    // }],
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
        for id in KnownCargoOptions::get_profile_ids() {
            assert!(!KnownCargoOptions::get_profile_possible_values(id).is_empty());
        }
    }
}
