use std::fmt::{Display, Formatter};

use anyhow::Context;
use clap::ValueEnum;
use console::{style, Style};
use inquire::{Confirm, min_length, Select, Text};
use inquire::ui::{Color, RenderConfig};
use similar::ChangeTag;

use cargo_wizard::{parse_manifest, ParsedManifest, resolve_manifest_path};

use crate::cli::PredefinedTemplate;

fn template_style() -> Style {
    Style::new().cyan()
}

fn profile_style() -> Style {
    Style::new().green()
}

pub fn dialog_root() -> anyhow::Result<()> {
    let template = dialog_template()?;
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let manifest = parse_manifest(&manifest_path)?;
    let profile = dialog_profile(&manifest)?;

    if let Some(manifest) = dialog_apply_diff(manifest, &profile, template.clone())? {
        manifest.write(&manifest_path)?;

        println!(
            "Template {} applied to profile {}.",
            template_style().apply_to(match template {
                PredefinedTemplate::FastCompile => "FastCompile",
                PredefinedTemplate::FastRuntime => "FastRuntime",
                PredefinedTemplate::MinSize => "MinSize",
            }),
            profile_style().apply_to(profile)
        );

        if let PredefinedTemplate::FastRuntime = template {
            println!(
                "Consider also using the {} subcommand to further optimize your binary.",
                Style::new().yellow().apply_to("cargo-pgo")
            );
        }
    }

    Ok(())
}

fn dialog_apply_diff(
    manifest: ParsedManifest,
    profile: &str,
    template: PredefinedTemplate,
) -> anyhow::Result<Option<ParsedManifest>> {
    let orig_manifest = manifest.clone();
    let orig_profile_text = orig_manifest
        .get_profile(profile)
        .map(|t| t.to_string())
        .unwrap_or_default();

    let manifest = manifest.apply_template(profile, template.resolve_to_template())?;
    let new_profile_text = manifest
        .get_profile(profile)
        .map(|t| t.to_string())
        .unwrap_or_default();

    let diff = calculate_diff(&orig_profile_text, &new_profile_text);
    println!("{diff}");

    let answer = Confirm::new(&format!(
        "Do you want to apply the above diff to the {} profile?",
        profile_style().apply_to(profile)
    ))
    .with_default(true)
    .prompt()
    .context("Cannot confirm diff")?;

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
                Profile::CreateNew => f.write_str("<Create a new profile>"),
            }
        }
    }

    let mut profiles = vec![Profile::Dev, Profile::Release];
    let mut original_profiles: Vec<_> = manifest
        .get_original_profiles()
        .iter()
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
    .with_render_config(profile_render_config())
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
    Ok(Text::new("Select profile name:")
        .with_validator(min_length!(1))
        .with_render_config(profile_render_config())
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
    .with_render_config(template_render_config())
    .prompt()
    .context("Cannot select template")?;

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
