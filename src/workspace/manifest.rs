use std::path::{Path, PathBuf};

use anyhow::Context;
use toml_edit::{Document, table, value};

use crate::toml::{TableItem, TomlTableTemplate};

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

#[derive(Clone)]
pub enum BuiltinProfile {
    Dev,
    Release,
}

#[derive(Clone)]
pub struct TomlProfileTemplate {
    pub inherits: BuiltinProfile,
    pub template: TomlTableTemplate,
}

/// Manifest parsed out of a Cargo.toml file.
#[derive(Clone)]
pub struct CargoManifest {
    path: PathBuf,
    document: Document,
}

impl CargoManifest {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let manifest = std::fs::read_to_string(path).context("Cannot read Cargo.toml manifest")?;
        let document = manifest
            .parse::<Document>()
            .context("Cannot parse Cargo.toml manifest")?;
        Ok(Self {
            document,
            path: path.to_path_buf(),
        })
    }

    pub fn get_path(&self) -> &Path {
        &self.path
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
        name: &str,
        template: TomlProfileTemplate,
    ) -> anyhow::Result<Self> {
        let profiles_table = self
            .document
            .entry("profile")
            .or_insert(table())
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("The profile item in Cargo.toml is not a table"))?;
        profiles_table.set_dotted(true);

        let profile_table = profiles_table
            .entry(name)
            .or_insert(table())
            .as_table_mut()
            .ok_or_else(|| {
                anyhow::anyhow!("The profile.{name} table in Cargo.toml is not a table")
            })?;

        let mut values = template.template.items.clone();

        if !is_builtin_profile(name) {
            let inherits = match template.inherits {
                BuiltinProfile::Dev => "dev",
                BuiltinProfile::Release => "release",
            };

            // Add "inherits" to the table
            values.insert(0, TableItem::string("inherits", inherits));
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

        Ok(self)
    }

    pub fn write(self) -> anyhow::Result<()> {
        std::fs::write(self.path, self.document.to_string())
            .context("Cannot write Cargo.toml manifest")?;
        Ok(())
    }
}

fn is_builtin_profile(name: &str) -> bool {
    matches!(name, "dev" | "release")
}
