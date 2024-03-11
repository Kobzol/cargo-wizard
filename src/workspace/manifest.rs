use std::path::{Path, PathBuf};

use anyhow::Context;
use toml_edit::{table, value, Array, Document, Item, Value};

use crate::template::{dev_profile, release_profile, TemplateItemId};
use crate::{Template, TomlValue};

/// Tries to resolve the workspace root manifest (Cargo.toml) path from the current directory.
pub fn resolve_manifest_path() -> anyhow::Result<PathBuf> {
    let cmd = cargo_metadata::MetadataCommand::new();
    let metadata = cmd
        .exec()
        .map_err(|error| anyhow::anyhow!("Cannot get cargo metadata: {:?}", error))?;
    let manifest_path = metadata
        .workspace_root
        .into_std_path_buf()
        .join("Cargo.toml");
    Ok(manifest_path)
}

#[derive(Clone, Copy, Debug)]
pub enum BuiltinProfile {
    Dev,
    Release,
}

impl BuiltinProfile {
    fn name(&self) -> &str {
        match self {
            BuiltinProfile::Dev => "dev",
            BuiltinProfile::Release => "release",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Profile {
    Builtin(BuiltinProfile),
    Custom(String),
}

impl Profile {
    pub fn dev() -> Self {
        Self::Builtin(BuiltinProfile::Dev)
    }

    pub fn release() -> Self {
        Self::Builtin(BuiltinProfile::Release)
    }

    pub fn name(&self) -> &str {
        match self {
            Profile::Builtin(builtin) => builtin.name(),
            Profile::Custom(name) => name.as_str(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Profile::Builtin(_))
    }
}

/// Manifest parsed out of a `Cargo.toml` file.
#[derive(Clone)]
pub struct CargoManifest {
    path: PathBuf,
    document: Document,
}

impl CargoManifest {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let manifest = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read Cargo.toml manifest from {}", path.display()))?;
        let document = manifest
            .parse::<Document>()
            .with_context(|| format!("Cannot parse Cargo.toml manifest from {}", path.display()))?;
        Ok(Self {
            document,
            path: path.to_path_buf(),
        })
    }

    pub fn get_profiles(&self) -> Vec<String> {
        self.document
            .get("profile")
            .and_then(|p| p.as_table_like())
            .map(|t| t.iter().map(|(name, _)| name.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn get_text(&self) -> String {
        self.document.to_string()
    }

    pub fn apply_template(
        mut self,
        profile: &Profile,
        template: &Template,
    ) -> anyhow::Result<Self> {
        let profiles_table = self
            .document
            .entry("profile")
            .or_insert(table())
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("The profile item in Cargo.toml is not a table"))?;
        profiles_table.set_dotted(true);

        let profile_table = profiles_table
            .entry(profile.name())
            .or_insert(table())
            .as_table_mut()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "The profile.{} table in Cargo.toml is not a table",
                    profile.name()
                )
            })?;

        // If we're applying the template to a built-in profile (dev or release), we skip the items
        // that still have the default value.
        // However, we don't do that for custom profiles based on dev/release, since dev/release
        // might not actually contain the default values in that case.
        let base_template = if profile.is_builtin() {
            Some(match template.inherits() {
                BuiltinProfile::Dev => dev_profile().build(),
                BuiltinProfile::Release => release_profile().build(),
            })
        } else {
            None
        };
        let mut values: Vec<_> = template
            .iter_items()
            .filter_map(|(id, value)| {
                let Some(name) = id_to_item_name(id) else {
                    return None;
                };

                // Check if there is any existing value in the TOML profile table
                let existing_value = profile_table.get(name).and_then(|item| {
                    if let Some(value) = item.as_bool() {
                        Some(TomlValue::Bool(value))
                    } else if let Some(value) = item.as_integer() {
                        Some(TomlValue::Int(value))
                    } else if let Some(value) = item.as_str() {
                        Some(TomlValue::String(value.to_string()))
                    } else {
                        None
                    }
                });
                // Check if we modify a built-in profile, and if we have a default vaule for this
                // item in the profile.
                let default_item = base_template.as_ref().and_then(|t| t.get_item(id).cloned());

                // If we have the same value as the default, and the existing value also matches the
                // default, skip this item.
                let base_item = existing_value.or(default_item);
                if let Some(base_value) = base_item {
                    if &base_value == value {
                        return None;
                    }
                };

                Some(TableItem {
                    name: name.to_string(),
                    value: value.clone(),
                })
            })
            .collect();

        if !profile.is_builtin() {
            // Add "inherits" to the table
            values.insert(0, TableItem::string("inherits", template.inherits().name()));
        }

        for entry in values {
            let mut new_value = entry.value.to_toml_value();

            if let Some(existing_item) = profile_table.get_mut(&entry.name) {
                if let Some(value) = existing_item.as_value() {
                    *new_value.decor_mut() = value.decor().clone();
                }
                *existing_item = value(new_value);
            } else {
                profile_table.insert(&entry.name, value(new_value));
            }
        }

        // Add necessary Cargo features
        if template.get_item(TemplateItemId::CodegenBackend).is_some() {
            if let Some(features) = self
                .document
                .entry("cargo-features")
                .or_insert(Item::Value(Value::Array(Array::new())))
                .as_array_mut()
            {
                if !features
                    .iter()
                    .any(|v| v.as_str() == Some("codegen-backend"))
                {
                    features.push("codegen-backend");
                }
            }
        }

        Ok(self)
    }

    pub fn write(self) -> anyhow::Result<()> {
        std::fs::write(self.path, self.document.to_string())
            .context("Cannot write Cargo.toml manifest")?;
        Ok(())
    }
}

fn id_to_item_name(id: TemplateItemId) -> Option<&'static str> {
    match id {
        TemplateItemId::DebugInfo => Some("debug"),
        TemplateItemId::SplitDebugInfo => Some("split-debuginfo"),
        TemplateItemId::Strip => Some("strip"),
        TemplateItemId::Lto => Some("lto"),
        TemplateItemId::CodegenUnits => Some("codegen-units"),
        TemplateItemId::Panic => Some("panic"),
        TemplateItemId::OptimizationLevel => Some("opt-level"),
        TemplateItemId::CodegenBackend => Some("codegen-backend"),
        TemplateItemId::Incremental => Some("incremental"),
        TemplateItemId::TargetCpuInstructionSet
        | TemplateItemId::FrontendThreads
        | TemplateItemId::Linker => None,
    }
}

#[derive(Clone, Debug)]
struct TableItem {
    name: String,
    value: TomlValue,
}

impl TableItem {
    fn string(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: TomlValue::String(value.to_string()),
        }
    }
}
