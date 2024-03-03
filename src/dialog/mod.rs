use std::fmt::{Display, Formatter};

use anyhow::Context;
use clap::ValueEnum;
use inquire::{min_length, Select, Text};

use cargo_wizard::{parse_manifest, ParsedManifest, resolve_manifest_path};

use crate::cli::PredefinedTemplate;

pub fn dialog_root() -> anyhow::Result<()> {
    let template = dialog_template()?;
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let manifest = parse_manifest(&manifest_path)?;
    let profile = dialog_profile(&manifest)?;
    let manifest = manifest.apply_profile(&profile, template.resolve_to_template())?;
    manifest.write(&manifest_path)?;

    println!(
        "Template {} applied to profile {profile}",
        match template {
            PredefinedTemplate::FastCompile => "FastCompile",
            PredefinedTemplate::FastRuntime => "FastRuntime",
            PredefinedTemplate::MinSize => "MinSize",
        }
    );

    Ok(())
}

fn dialog_profile(manifest: &ParsedManifest) -> anyhow::Result<String> {
    enum Profile {
        Dev,
        Release,
        Custom(String),
        CreateNew,
    }
    impl Display for Profile {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Profile::Dev => f.write_str("dev (builtin)"),
                Profile::Release => f.write_str("release (builtin)"),
                Profile::Custom(name) => f.write_str(name),
                Profile::CreateNew => f.write_str("Create a new profile"),
            }
        }
    }

    let mut profiles = vec![Profile::Dev, Profile::Release];
    let mut original_profiles: Vec<_> = manifest
        .get_original_profiles()
        .keys()
        .filter(|p| !matches!(p.as_str(), "dev" | "release"))
        .cloned()
        .collect();
    original_profiles.sort();
    profiles.extend(original_profiles.into_iter().map(Profile::Custom));
    profiles.push(Profile::CreateNew);

    let selected = Select::new(
        "Select the profile that you want to update/create:",
        profiles,
    )
    .prompt()
    .context("Cannot select template")?;

    let profile = match selected {
        Profile::Dev => "dev".to_string(),
        Profile::Release => "release".to_string(),
        Profile::Custom(name) => name,
        Profile::CreateNew => dialog_profile_name()?,
    };

    Ok(profile)
}

fn dialog_profile_name() -> anyhow::Result<String> {
    Ok(Text::new("Select profile name")
        .with_validator(min_length!(1))
        .prompt()?)
}

fn dialog_template() -> anyhow::Result<PredefinedTemplate> {
    struct Template(PredefinedTemplate);

    impl Display for Template {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let msg = match self.0 {
                PredefinedTemplate::FastCompile => "FastCompile: minimize compile times",
                PredefinedTemplate::FastRuntime => "FastRuntime: maximize runtime performance",
                PredefinedTemplate::MinSize => "MinSize: minimize binary size",
            };
            f.write_str(msg)
        }
    }

    let selected = Select::new(
        "Select the template that you want to apply:",
        PredefinedTemplate::value_variants()
            .iter()
            .map(|template| Template(template.clone()))
            .collect(),
    )
    .prompt()
    .context("Cannot select template")?;

    Ok(selected.0)
}
