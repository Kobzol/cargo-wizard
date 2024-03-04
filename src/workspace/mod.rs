use std::path::Path;

use anyhow::Context;

use manifest::CargoManifest;

use crate::workspace::config::{config_path_from_manifest_path, CargoConfig};

pub mod config;
pub mod manifest;

pub struct CargoWorkspace {
    pub manifest: CargoManifest,
    pub config: Option<CargoConfig>,
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
