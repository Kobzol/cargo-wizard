use std::path::Path;

use anyhow::Context;

use crate::Template;
use manifest::CargoManifest;

use crate::workspace::config::{config_path_from_manifest_path, CargoConfig};
use crate::workspace::manifest::Profile;

pub mod config;
pub mod manifest;

/// Cargo workspace project.
#[derive(Clone)]
pub struct CargoWorkspace {
    manifest: CargoManifest,
    config: CargoConfig,
}

impl CargoWorkspace {
    pub fn apply_template(
        self,
        profile: &Profile,
        template: &Template,
    ) -> anyhow::Result<ModifiedWorkspace> {
        let old_manifest = self.manifest.clone();
        let new_manifest = self.manifest.apply_template(profile, &template)?;
        let manifest = if old_manifest.get_text() == new_manifest.get_text() {
            ModificationResult::NoChange
        } else {
            ModificationResult::Modified {
                old: old_manifest,
                new: new_manifest,
            }
        };

        let old_config = self.config.clone();
        let new_config = self.config.apply_template(&template)?;
        let config = if old_config.get_text() == new_config.get_text() {
            ModificationResult::NoChange
        } else {
            ModificationResult::Modified {
                old: old_config,
                new: new_config,
            }
        };
        Ok(ModifiedWorkspace { manifest, config })
    }

    pub fn existing_profiles(&self) -> Vec<String> {
        self.manifest.get_profiles()
    }
}

/// Workspace that was modified through a template.
pub struct ModifiedWorkspace {
    manifest: ModificationResult<CargoManifest>,
    config: ModificationResult<CargoConfig>,
}

impl ModifiedWorkspace {
    pub fn manifest(&self) -> &ModificationResult<CargoManifest> {
        &self.manifest
    }

    pub fn config(&self) -> &ModificationResult<CargoConfig> {
        &self.config
    }

    pub fn write(self) -> anyhow::Result<()> {
        match self.manifest {
            ModificationResult::NoChange => {}
            ModificationResult::Modified { new, .. } => {
                new.write()?;
            }
        }
        match self.config {
            ModificationResult::NoChange => {}
            ModificationResult::Modified { new, .. } => {
                new.write()?;
            }
        }
        Ok(())
    }
}

/// Result of modification of a manifest or a config.
pub enum ModificationResult<T> {
    NoChange,
    Modified { old: T, new: T },
}

/// Parses a Cargo workspace from a Cargo.toml manifest path.
pub fn parse_workspace(manifest_path: &Path) -> anyhow::Result<CargoWorkspace> {
    let manifest = CargoManifest::from_path(manifest_path)?;
    let config = Some(config_path_from_manifest_path(manifest_path))
        .filter(|p| p.exists())
        .map(|path| CargoConfig::from_path(&path))
        .transpose()
        .with_context(|| "Cannot load config.toml")?
        .unwrap_or_else(|| CargoConfig::empty_from_manifest(manifest_path));

    Ok(CargoWorkspace { manifest, config })
}
