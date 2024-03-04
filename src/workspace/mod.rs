use std::path::Path;

use manifest::CargoManifest;

pub mod manifest;

pub struct CargoWorkspace {
    pub manifest: CargoManifest,
}

/// Parses a Cargo workspace from a Cargo.tom manifest path.
pub fn parse_workspace(manifest_path: &Path) -> anyhow::Result<CargoWorkspace> {
    let manifest = CargoManifest::from_path(manifest_path)?;
    Ok(CargoWorkspace { manifest })
}
