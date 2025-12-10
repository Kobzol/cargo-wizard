//! `cargo-wizard` is a Cargo subcommand that can apply preconfigured templates to your Cargo.toml manifest.
//!
//! Non-interactive command-line usage:
//! ```bash
//! cargo wizard apply <template> <profile> [--nightly=on]
//! ```
//! Interactive command-line usage:
//! ```bash
//! cargo wizard
//! ```
//!
//! You can also use this crate as a library, although it probably won't be very useful.

pub use predefined::*;
pub use template::{Template, TemplateItemId, WizardOptions};
pub use toml::TomlValue;
pub use utils::get_core_count;
pub use workspace::config::CargoConfig;
pub use workspace::manifest::{BuiltinProfile, CargoManifest, Profile, resolve_manifest_path};
pub use workspace::{CargoWorkspace, ModificationResult, ModifiedWorkspace, parse_workspace};

mod predefined;
mod template;
mod toml;
mod utils;
mod workspace;
