use std::fmt::{Display, Formatter};

use anyhow::Context;
use clap::ValueEnum;
use console::{style, Style};
use inquire::{Confirm, min_length, Select, Text};
use inquire::ui::{Color, RenderConfig};
use similar::ChangeTag;

use cargo_wizard::{CargoConfig, CargoManifest, parse_workspace, resolve_manifest_path};
use cargo_wizard::PredefinedTemplateKind;
pub use error::{DialogError, DialogResult};

mod error;

pub fn dialog_root() -> DialogResult<()> {
    let template_kind = dialog_template()?;
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let workspace = parse_workspace(&manifest_path)?;
    let profile = dialog_profile(&workspace.manifest)?;

    if let Some(manifest) = dialog_apply_diff(workspace.manifest, &profile, template_kind.clone())?
    {
        manifest.write()?;

        if let Some(config_template) = template_kind.build_template().config {
            let config = workspace
                .config
                .unwrap_or_else(|| CargoConfig::empty_from_manifest(&manifest_path));
            let config = config
                .apply_template(config_template)
                .context("Cannot apply config.toml template")?;
            config.write()?;
        }

        println!(
            "✅ Template {} applied to profile {}.",
            template_style().apply_to(match template_kind {
                PredefinedTemplateKind::FastCompile => "FastCompile",
                PredefinedTemplateKind::FastRuntime => "FastRuntime",
                PredefinedTemplateKind::MinSize => "MinSize",
            }),
            profile_style().apply_to(&profile)
        );

        let profile_flag = match profile.as_str() {
            "dev" => None,
            "release" => Some("--release".to_string()),
            profile => Some(format!("--profile={profile}")),
        };
        if let Some(flag) = profile_flag {
            println!(
                "❗ Do not forget to run `{}` to use the selected profile.",
                command_style().apply_to(format!("cargo <cmd> {flag}"))
            );
        }

        if let PredefinedTemplateKind::FastRuntime = template_kind {
            println!(
                "\nTip: Consider using the {} subcommand to further optimize your binary.",
                command_style().apply_to("cargo-pgo")
            );
        }
    }

    Ok(())
}

fn dialog_apply_diff(
    manifest: CargoManifest,
    profile: &str,
    template_kind: PredefinedTemplateKind,
) -> DialogResult<Option<CargoManifest>> {
    let orig_manifest = manifest.clone();
    let orig_profile_text = orig_manifest
        .get_profile(profile)
        .map(|t| t.to_string())
        .unwrap_or_default();

    let template = template_kind.build_template();
    let manifest = manifest.apply_template(profile, template.profile)?;
    let new_profile_text = manifest
        .get_profile(profile)
        .map(|t| t.to_string())
        .unwrap_or_default();

    let diff = calculate_diff(&orig_profile_text, &new_profile_text);
    println!("\r{diff}");

    let answer = Confirm::new(&format!(
        "Do you want to apply the above diff to the {} profile?",
        profile_style().apply_to(profile)
    ))
    .with_default(true)
    .prompt()?;

    Ok(answer.then_some(manifest))
}

// Taken from https://github.com/mitsuhiko/similar/blob/main/examples/terminal-inline.rs
fn calculate_diff(original: &str, new: &str) -> String {
    use std::fmt::Write;

    struct Line(Option<usize>);

    impl Display for Line {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            match self.0 {
                None => write!(f, "    "),
                Some(idx) => write!(f, "{:<4}", idx + 1),
            }
        }
    }

    let diff = similar::TextDiff::from_lines(original, new);
    let mut output = String::new();
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            write!(output, "{:-^1$}", "-", 80).unwrap();
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    output,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )
                .unwrap();
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(output, "{}", s.apply_to(value).underlined().on_black()).unwrap();
                    } else {
                        write!(output, "{}", s.apply_to(value)).unwrap();
                    }
                }
                if change.missing_newline() {
                    writeln!(output).unwrap();
                }
            }
        }
    }
    output
}

fn dialog_profile(manifest: &CargoManifest) -> DialogResult<String> {
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
                Profile::CreateNew => f.write_str("<Create a new profile>"),
            }
        }
    }

    let mut profiles = vec![Profile::Dev, Profile::Release];
    let mut original_profiles: Vec<_> = manifest
        .get_profiles()
        .into_iter()
        .filter(|p| !matches!(p.as_str(), "dev" | "release"))
        .collect();
    original_profiles.sort();
    profiles.extend(original_profiles.into_iter().map(Profile::Custom));
    profiles.push(Profile::CreateNew);

    let selected = Select::new(
        "Select the profile that you want to update/create:",
        profiles,
    )
    .with_render_config(profile_render_config())
    .prompt()?;

    let profile = match selected {
        Profile::Dev => "dev".to_string(),
        Profile::Release => "release".to_string(),
        Profile::Custom(name) => name,
        Profile::CreateNew => dialog_profile_name()?,
    };

    Ok(profile)
}

fn dialog_profile_name() -> DialogResult<String> {
    Ok(Text::new("Select profile name:")
        .with_validator(min_length!(1))
        .with_render_config(profile_render_config())
        .prompt()?)
}

fn dialog_template() -> DialogResult<PredefinedTemplateKind> {
    struct Template(PredefinedTemplateKind);

    impl Display for Template {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let msg = match self.0 {
                PredefinedTemplateKind::FastCompile => "FastCompile: minimize compile times",
                PredefinedTemplateKind::FastRuntime => "FastRuntime: maximize runtime performance",
                PredefinedTemplateKind::MinSize => "MinSize: minimize binary size",
            };
            f.write_str(msg)
        }
    }

    let selected = Select::new(
        "Select the template that you want to apply:",
        PredefinedTemplateKind::value_variants()
            .iter()
            .map(|template| Template(template.clone()))
            .collect(),
    )
    .with_render_config(template_render_config())
    .prompt()?;

    Ok(selected.0)
}

fn template_render_config() -> RenderConfig<'static> {
    let mut render_config = RenderConfig::default_colored();
    render_config.answer = render_config.option.with_fg(Color::DarkCyan);
    render_config.selected_option = render_config
        .selected_option
        .map(|s| s.with_fg(Color::DarkCyan));
    render_config
}

fn profile_render_config() -> RenderConfig<'static> {
    let mut render_config = RenderConfig::default_colored();
    render_config.answer = render_config.option.with_fg(Color::DarkGreen);
    render_config.selected_option = render_config
        .selected_option
        .map(|s| s.with_fg(Color::DarkGreen));
    render_config
}

fn template_style() -> Style {
    Style::new().cyan()
}

fn profile_style() -> Style {
    Style::new().green()
}

fn command_style() -> Style {
    Style::new().yellow()
}
