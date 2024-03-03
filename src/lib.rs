//! `cargo-wizard` is a Cargo subcommand that can apply preconfigured templates to your Cargo.toml manifest.
//!
//! Command-line usage:
//! ```bash
//! cargo wizard apply <profile> <template>
//! ```
//!
//! You can also use this crate as a library, although it probably won't be very useful.

pub use manifest::{parse_manifest, resolve_manifest_path};
pub use templates::*;
pub use toml::TomlProfileTemplate;

mod manifest;
mod templates;
mod toml;
