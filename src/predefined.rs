use rustc_version::Version;

use crate::template::{dev_profile, release_profile, TemplateItemId};
use crate::toml::TomlValue;
use crate::utils::get_core_count;
use crate::{Template, WizardOptions};

/// Enumeration of predefined templates.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum PredefinedTemplateKind {
    /// Profile designed for fast compilation times.
    FastCompile,
    /// Profile designed for fast runtime performance.
    FastRuntime,
    /// Profile designed for minimal binary size.
    MinSize,
}

impl PredefinedTemplateKind {
    pub fn build_template(&self, options: &WizardOptions) -> Template {
        match self {
            PredefinedTemplateKind::FastCompile => fast_compile_template(options),
            PredefinedTemplateKind::FastRuntime => fast_runtime_template(),
            PredefinedTemplateKind::MinSize => min_size_template(),
        }
    }
}

/// Template that focuses on quick compile time.
pub fn fast_compile_template(options: &WizardOptions) -> Template {
    let mut builder = dev_profile().item(TemplateItemId::DebugInfo, TomlValue::int(0));

    #[cfg(unix)]
    match rustc_version::version_meta() {
        Ok(meta) => {
            if (meta.semver < Version::new(1, 90, 0) || &meta.host != "x86_64-unknown-linux-gnu")
                && &meta.host != "aarch64-apple-darwin"
            {
                builder = builder.item(TemplateItemId::Linker, TomlValue::string("lld"));
            }
        }
        Err(error) => {
            builder = builder.item(TemplateItemId::Linker, TomlValue::string("lld"));
            eprintln!("Cannot get compiler version. ({error:?})");
        }
    }

    if options.nightly_items_enabled() {
        builder = builder
            .item(
                TemplateItemId::CodegenBackend,
                TomlValue::string("cranelift"),
            )
            .item(
                TemplateItemId::FrontendThreads,
                TomlValue::Int(get_core_count()),
            )
    }
    builder.build()
}

/// Template that focuses on maximum runtime performance.
pub fn fast_runtime_template() -> Template {
    release_profile()
        .item(TemplateItemId::Lto, TomlValue::bool(true))
        .item(TemplateItemId::CodegenUnits, TomlValue::int(1))
        .item(TemplateItemId::Panic, TomlValue::string("abort"))
        .item(
            TemplateItemId::TargetCpuInstructionSet,
            TomlValue::string("native"),
        )
        .build()
}

/// Template that template focuses on minimal binary size.
pub fn min_size_template() -> Template {
    release_profile()
        .item(TemplateItemId::DebugInfo, TomlValue::bool(false))
        .item(TemplateItemId::Strip, TomlValue::bool(true))
        .item(TemplateItemId::Lto, TomlValue::bool(true))
        .item(TemplateItemId::OptimizationLevel, TomlValue::string("z"))
        .item(TemplateItemId::CodegenUnits, TomlValue::int(1))
        .item(TemplateItemId::Panic, TomlValue::string("abort"))
        .build()
}

/// Test that the predefined templates can be created without panicking.
#[cfg(test)]
mod tests {
    use crate::{fast_compile_template, fast_runtime_template, min_size_template, WizardOptions};

    #[test]
    fn create_fast_compile_template() {
        fast_compile_template(&WizardOptions::default());
    }

    #[test]
    fn create_fast_runtime_template() {
        fast_runtime_template();
    }

    #[test]
    fn create_min_size_template() {
        min_size_template();
    }
}
