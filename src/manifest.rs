use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use toml_edit::{Document, Item, table, value};

use crate::toml::{TemplateEntry, TomlTableTemplate};

#[derive(Debug)]
pub struct ParsedProfile {
    name: String,
    items: HashMap<String, Item>,
}

/// Manifest parsed out of a Cargo.toml file.
#[derive(Debug)]
pub struct ParsedManifest {
    document: Document,
    /// Original profiles present in the manifest, e.g. `[profile.dev]`.
    profiles: HashMap<String, ParsedProfile>,
}

impl ParsedManifest {
    pub fn get_original_profiles(&self) -> &HashMap<String, ParsedProfile> {
        &self.profiles
    }

    pub fn apply_profile(
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

        let mut values = template.template.fields.clone();

        if !is_builtin_profile(name) {
            let inherits = match template.inherits {
                BuiltinProfile::Dev => "dev",
                BuiltinProfile::Release => "release",
            };

            // Add "inherits" to the table
            values.insert(0, TemplateEntry::string("inherits", inherits));
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

    pub fn write(self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, self.document.to_string())?;
        Ok(())
    }
}

/// Parses a Cargo.toml manifest from disk.
pub fn parse_manifest(path: &Path) -> anyhow::Result<ParsedManifest> {
    let manifest = std::fs::read_to_string(path).context("Cannot read Cargo.toml manifest")?;
    let manifest = manifest
        .parse::<Document>()
        .context("Cannot parse Cargo.toml manifest")?;

    let profiles = if let Some(profiles) = manifest.get("profile").and_then(|p| p.as_table_like()) {
        profiles
            .iter()
            .filter_map(|(name, table)| table.as_table().map(|t| (name, t)))
            .map(|(name, table)| {
                let name = name.to_string();

                let items = table
                    .iter()
                    .map(|(name, item)| (name.to_string(), item.clone()))
                    .collect();

                let profile = ParsedProfile {
                    name: name.clone(),
                    items,
                };

                (name, profile)
            })
            .collect()
    } else {
        Default::default()
    };
    Ok(ParsedManifest {
        profiles,
        document: manifest,
    })
}

pub fn is_builtin_profile(name: &str) -> bool {
    matches!(name, "dev" | "release")
}

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

pub enum BuiltinProfile {
    Dev,
    Release,
}

pub struct TomlProfileTemplate {
    pub inherits: BuiltinProfile,
    pub template: TomlTableTemplate,
}
