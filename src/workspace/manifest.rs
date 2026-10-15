use std::path::{Path, PathBuf};

use anyhow::Context;
use toml_edit::{Array, DocumentMut, Item, TableLike, Value, table, value};

use crate::template::{TemplateBuilder, TemplateItemId, dev_profile, release_profile};
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl From<BuiltinProfile> for Profile {
    fn from(profile: BuiltinProfile) -> Self {
        Self::Builtin(profile)
    }
}

/// Manifest parsed out of a `Cargo.toml` file.
#[derive(Clone)]
pub struct CargoManifest {
    path: PathBuf,
    document: DocumentMut,
}

impl CargoManifest {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let manifest = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read Cargo.toml manifest from {}", path.display()))?;
        let document = manifest
            .parse::<DocumentMut>()
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

    fn profile_table(&self, profile: &Profile) -> Option<&dyn TableLike> {
        self.document
            .get("profile")?
            .as_table_like()?
            .get(profile.name())?
            .as_table_like()
    }

    /// Loads the supported, explicitly configured values for a profile.
    pub fn load_template(&self, profile: &Profile) -> anyhow::Result<Template> {
        let table = self.profile_table(profile);
        let inherits = match (profile, table) {
            (Profile::Builtin(profile), _) => (*profile).into(),
            (Profile::Custom(_), None) => Profile::dev(),
            (Profile::Custom(name), Some(table)) => {
                match table.get("inherits").and_then(Item::as_str) {
                    Some("dev") => Profile::dev(),
                    Some("release") => Profile::release(),
                    Some(custom) => Profile::Custom(custom.to_string()),
                    None => anyhow::bail!(
                        "The profile.{name}.inherits item in Cargo.toml must be a string"
                    ),
                }
            }
        };
        let mut template = TemplateBuilder::new(inherits).build();
        for (id, name) in PROFILE_ITEMS {
            let Some(item) = table.and_then(|table| table.get(name)) else {
                continue;
            };
            let value = item_to_toml_value(item).ok_or_else(|| {
                anyhow::anyhow!(
                    "The profile.{}.{name} item in Cargo.toml must be a boolean, integer, or string",
                    profile.name()
                )
            })?;
            template.insert_item(id, value);
        }
        Ok(template)
    }

    /// Sets the template's profile items. With `replace`, also removes the supported items that the
    /// template does not set.
    pub fn apply_template(
        mut self,
        profile: &Profile,
        template: &Template,
        replace: bool,
    ) -> anyhow::Result<Self> {
        let existing_table = self.profile_table(profile);
        let existing_value = |name: &str| {
            existing_table
                .and_then(|table| table.get(name))
                .and_then(item_to_toml_value)
        };

        // For a built-in destination, omit settings that already match that profile's Cargo
        // defaults. Keep custom profiles explicit because their inherited profile may be
        // overridden elsewhere in this manifest.
        let base_template = match profile {
            Profile::Builtin(BuiltinProfile::Dev) => Some(dev_profile().build()),
            Profile::Builtin(BuiltinProfile::Release) => Some(release_profile().build()),
            Profile::Custom(_) => None,
        };
        let mut values: Vec<(&str, TomlValue)> = template
            .iter_items()
            .filter_map(|(id, value)| {
                let name = id_to_item_name(id)?;
                // Skip the item if it already has this value, explicitly or by default.
                let base_value =
                    existing_value(name).or_else(|| base_template.as_ref()?.get_item(id).cloned());
                (base_value.as_ref() != Some(value)).then(|| (name, value.clone()))
            })
            .collect();
        if !profile.is_builtin() {
            let inherits = TomlValue::string(template.inherits().name());
            if existing_value("inherits") != Some(inherits.clone()) {
                values.insert(0, ("inherits", inherits));
            }
        }
        let removals: Vec<&str> = PROFILE_ITEMS
            .iter()
            .filter(|(id, name)| {
                replace
                    && template.get_item(*id).is_none()
                    && existing_table.is_some_and(|table| table.contains_key(name))
            })
            .map(|(_, name)| *name)
            .collect();
        if values.is_empty() && removals.is_empty() {
            return Ok(self);
        }

        let profiles_item = self.document.entry("profile").or_insert(table());
        let profiles_are_regular_table = profiles_item.is_table();
        let profiles_table = profiles_item
            .as_table_like_mut()
            .ok_or_else(|| anyhow::anyhow!("The profile item in Cargo.toml is not a table"))?;
        if profiles_are_regular_table {
            profiles_table.set_dotted(true);
        }

        let profile_table = profiles_table
            .entry(profile.name())
            .or_insert(table())
            .as_table_like_mut()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "The profile.{} table in Cargo.toml is not a table",
                    profile.name()
                )
            })?;

        for (name, new_value) in values {
            let mut new_value = new_value.to_toml_value();

            if let Some(existing_item) = profile_table.get_mut(name) {
                if let Some(value) = existing_item.as_value() {
                    *new_value.decor_mut() = value.decor().clone();
                }
                *existing_item = value(new_value);
            } else {
                profile_table.insert(name, value(new_value));
            }
        }
        for name in &removals {
            profile_table.remove(name);
        }

        // Add necessary Cargo features
        if template.get_item(TemplateItemId::CodegenBackend).is_some()
            && let Some(features) = self
                .document
                .entry("cargo-features")
                .or_insert(Item::Value(Value::Array(Array::new())))
                .as_array_mut()
            && !features
                .iter()
                .any(|v| v.as_str() == Some("codegen-backend"))
        {
            features.push("codegen-backend");
        }
        let codegen_backend_used = self
            .document
            .get("profile")
            .is_some_and(|profiles| table_contains_item(profiles, "codegen-backend"));
        if removals.contains(&"codegen-backend")
            && !codegen_backend_used
            && let Some(features) = self
                .document
                .get_mut("cargo-features")
                .and_then(Item::as_array_mut)
        {
            features.retain(|feature| feature.as_str() != Some("codegen-backend"));
        }

        Ok(self)
    }

    pub fn write(self) -> anyhow::Result<()> {
        std::fs::write(self.path, self.document.to_string())
            .context("Cannot write Cargo.toml manifest")?;
        Ok(())
    }
}

const PROFILE_ITEMS: [(TemplateItemId, &str); 9] = [
    (TemplateItemId::DebugInfo, "debug"),
    (TemplateItemId::SplitDebugInfo, "split-debuginfo"),
    (TemplateItemId::Strip, "strip"),
    (TemplateItemId::Lto, "lto"),
    (TemplateItemId::CodegenUnits, "codegen-units"),
    (TemplateItemId::Panic, "panic"),
    (TemplateItemId::OptimizationLevel, "opt-level"),
    (TemplateItemId::Incremental, "incremental"),
    (TemplateItemId::CodegenBackend, "codegen-backend"),
];

fn item_to_toml_value(item: &Item) -> Option<TomlValue> {
    if let Some(value) = item.as_bool() {
        Some(TomlValue::Bool(value))
    } else if let Some(value) = item.as_integer() {
        Some(TomlValue::Int(value))
    } else {
        item.as_str()
            .map(|value| TomlValue::String(value.to_string()))
    }
}

fn table_contains_item(item: &Item, name: &str) -> bool {
    item.as_table_like().is_some_and(|table| {
        table
            .iter()
            .any(|(key, item)| key == name || table_contains_item(item, name))
    })
}

fn id_to_item_name(id: TemplateItemId) -> Option<&'static str> {
    PROFILE_ITEMS
        .iter()
        .find(|(item_id, _)| *item_id == id)
        .map(|(_, name)| *name)
}
