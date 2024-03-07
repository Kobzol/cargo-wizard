use std::path::Path;

use anyhow::Context;

use crate::Template;
use manifest::CargoManifest;

use crate::workspace::config::{config_path_from_manifest_path, CargoConfig};

pub mod config;
pub mod manifest;

pub struct CargoWorkspace {
    pub manifest: CargoManifest,
    /// None means that the config was not found on disk
    /// If it is also None during [`CargoWorkspace::write`], then no config
    /// will be written to disk.
    pub config: Option<CargoConfig>,
}

impl CargoWorkspace {
    pub fn apply_template(mut self, profile: &str, template: Template) -> anyhow::Result<Self> {
        self.manifest = self.manifest.apply_template(profile, &template)?;

        let original_config = self
            .config
            .unwrap_or_else(|| CargoConfig::empty_from_manifest(&self.manifest.get_path()));

        let orig_config = original_config.clone();
        let modified_config = original_config.apply_template(&template)?;
        if orig_config.is_same_as(&modified_config) {
            self.config = None;
        } else {
            self.config = Some(modified_config);
        }

        Ok(self)
    }

    pub fn existing_profiles(&self) -> Vec<String> {
        self.manifest.get_profiles()
    }

    pub fn write(self) -> anyhow::Result<()> {
        self.manifest.write()?;
        if let Some(config) = self.config {
            config.write()?;
        }
        Ok(())
    }
}

/// Parses a Cargo workspace from a Cargo.toml manifest path.
pub fn parse_workspace(manifest_path: &Path) -> anyhow::Result<CargoWorkspace> {
    let manifest = CargoManifest::from_path(manifest_path)?;
    let config = Some(config_path_from_manifest_path(manifest_path))
        .filter(|p| p.exists())
        .map(|path| CargoConfig::from_path(&path))
        .transpose()
        .with_context(|| "Cannot load config.toml")?;

    Ok(CargoWorkspace { manifest, config })
}
