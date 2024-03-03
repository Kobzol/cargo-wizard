pub use manifest::parse_manifest;
pub use templates::fast_compile_template;
pub use toml::TomlProfileTemplate;

use crate::toml::TomlValue;

mod manifest;
mod templates;
mod toml;

struct ProfileData {
    items: Vec<(String, TomlValue)>,
}
