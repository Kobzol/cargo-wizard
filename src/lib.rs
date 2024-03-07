//! `cargo-wizard` is a Cargo subcommand that can apply preconfigured templates to your Cargo.toml manifest.
//!
//! Non-interactive command-line usage:
//! ```bash
//! cargo wizard apply <template> <profile>
//! ```
//! Interactive command-line usage:
//! ```bash
//! cargo wizard
//! ```
//!
//! You can also use this crate as a library, although it probably won't be very useful.

pub use predefined::*;
pub use template::{Template, TemplateItemId};
pub use toml::TomlValue;
pub use workspace::config::CargoConfig;
pub use workspace::manifest::{resolve_manifest_path, CargoManifest};
pub use workspace::{parse_workspace, CargoWorkspace, ModificationResult, ModifiedWorkspace};

mod predefined;
mod template;
mod toml;
mod workspace;
