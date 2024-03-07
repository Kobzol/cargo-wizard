//! `cargo-wizard` is a Cargo subcommand that can apply preconfigured templates to your Cargo.toml manifest.
//!
//! Command-line usage:
//! ```bash
//! cargo wizard apply <profile> <template>
//! ```
//!
//! You can also use this crate as a library, although it probably won't be very useful.

pub use predefined::*;
pub use template::{Template, TemplateBuilder, TemplateItemId};
pub use toml::TomlValue;
pub use workspace::config::CargoConfig;
pub use workspace::manifest::{resolve_manifest_path, CargoManifest, TomlProfileTemplate};
pub use workspace::{parse_workspace, CargoWorkspace, ModificationResult, ModifiedWorkspace};

mod predefined;
mod template;
mod toml;
mod workspace;
