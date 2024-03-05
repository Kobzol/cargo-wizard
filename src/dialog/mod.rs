use std::fmt::{Display, Formatter};

use anyhow::Context;
use clap::ValueEnum;
use console::{style, Style};
use inquire::{Confirm, min_length, Select, Text};
use inquire::ui::{Color, RenderConfig};
use similar::ChangeTag;

use cargo_wizard::{
    CargoConfig, CargoManifest, CargoWorkspace, parse_workspace, resolve_manifest_path, Template,
};
use cargo_wizard::PredefinedTemplateKind;
pub use error::{DialogError, DialogResult};

use crate::cli::CliConfig;

mod error;

pub fn dialog_root(cli_config: CliConfig) -> DialogResult<()> {
    let template_kind = dialog_template(&cli_config)?;
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let workspace = parse_workspace(&manifest_path)?;
    let profile = dialog_profile(&cli_config, &workspace.manifest)?;

    let template = template_kind.build_template();
    let diff_result = dialog_apply_diff(workspace, &profile, template)?;
    match diff_result {
        DiffPromptResponse::Accepted(workspace) => {
            workspace.write()?;

            on_template_applied(template_kind, &profile);
        }
        DiffPromptResponse::Denied => {}
        DiffPromptResponse::NoDiff => {
            println!("Nothing to apply, the profile already matched the template");
        }
    }

    Ok(())
}

fn on_template_applied(template: PredefinedTemplateKind, profile: &str) {
    clear_line();
    println!(
        "✅ Template {} applied to profile {}.",
        template_style().apply_to(match template {
            PredefinedTemplateKind::FastCompile => "FastCompile",
            PredefinedTemplateKind::FastRuntime => "FastRuntime",
            PredefinedTemplateKind::MinSize => "MinSize",
        }),
        profile_style().apply_to(&profile)
    );

    let profile_flag = match profile {
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

    if let PredefinedTemplateKind::FastRuntime = template {
        println!(
            "\nTip: Consider using the {} subcommand to further optimize your binary.",
            command_style().apply_to("cargo-pgo")
        );
    }
}

enum DiffPromptResponse {
    Accepted(CargoWorkspace),
    Denied,
    NoDiff,
}

fn dialog_apply_diff(
    mut workspace: CargoWorkspace,
    profile: &str,
    template: Template,
) -> DialogResult<DiffPromptResponse> {
    // Cargo.toml
    let orig_manifest_text = workspace.manifest.get_text();
    workspace.manifest = workspace
        .manifest
        .apply_template(profile, template.profile)?;
    let new_manifest_text = workspace.manifest.get_text();

    let manifest_diff = render_diff(&orig_manifest_text, &new_manifest_text);
    let manifest_changed = !manifest_diff.trim().is_empty();
    if manifest_changed {
        clear_line();
        println!("{}", file_style().apply_to("Cargo.toml"));
        println!("{manifest_diff}");
    }

    // .cargo/config.toml
    let config_diff = if let Some(config_template) = template.config {
        let config = workspace
            .config
            .unwrap_or_else(|| CargoConfig::empty_from_manifest(&workspace.manifest.get_path()));

        let old_config_text = config.get_text();
        let new_config = config
            .apply_template(config_template)
            .context("Cannot apply config.toml template")?;
        let new_manifest_text = new_config.get_text();
        let config_diff = render_diff(&old_config_text, &new_manifest_text);

        workspace.config = Some(new_config);
        if config_diff.trim().is_empty() {
            None
        } else {
            Some(config_diff)
        }
    } else {
        None
    };
    let config_changed = config_diff.is_some();
    if let Some(config_diff) = config_diff {
        clear_line();
        println!("{}", file_style().apply_to(".cargo/config.toml"));
        println!("{config_diff}");
    }

    if !manifest_changed && !config_changed {
        return Ok(DiffPromptResponse::NoDiff);
    }

    let multiple_diffs = manifest_changed && config_changed;

    let answer = Confirm::new(&format!(
        "Do you want to apply the above diff{}?",
        if multiple_diffs { "s" } else { "" }
    ))
    .with_default(true)
    .prompt()?;

    Ok(match answer {
        true => DiffPromptResponse::Accepted(workspace),
        false => DiffPromptResponse::Denied,
    })
}

// Taken from https://github.com/mitsuhiko/similar/blob/main/examples/terminal-inline.rs
fn render_diff(original: &str, new: &str) -> String {
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
                    ChangeTag::Equal => ("|", Style::new().dim()),
                };
                write!(
                    output,
                    "{}{} {} ",
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

fn dialog_profile(cli_config: &CliConfig, manifest: &CargoManifest) -> DialogResult<String> {
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
    .with_render_config(profile_render_config(cli_config))
    .prompt()?;

    let profile = match selected {
        Profile::Dev => "dev".to_string(),
        Profile::Release => "release".to_string(),
        Profile::Custom(name) => name,
        Profile::CreateNew => dialog_profile_name(cli_config)?,
    };

    Ok(profile)
}

fn dialog_profile_name(cli_config: &CliConfig) -> DialogResult<String> {
    Ok(Text::new("Select profile name:")
        .with_validator(min_length!(1))
        .with_render_config(profile_render_config(cli_config))
        .prompt()?)
}

fn dialog_template(cli_config: &CliConfig) -> DialogResult<PredefinedTemplateKind> {
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
    .with_render_config(template_render_config(cli_config))
    .prompt()?;

    Ok(selected.0)
}

fn template_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    let mut render_config = create_render_config(cli_config);
    render_config.answer = render_config.option.with_fg(Color::DarkCyan);
    render_config.selected_option = render_config
        .selected_option
        .map(|s| s.with_fg(Color::DarkCyan));
    render_config
}

fn profile_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    let mut render_config = create_render_config(cli_config);
    render_config.answer = render_config.option.with_fg(Color::DarkGreen);
    render_config.selected_option = render_config
        .selected_option
        .map(|s| s.with_fg(Color::DarkGreen));
    render_config
}

fn create_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    if cli_config.colors_enabled() {
        RenderConfig::default_colored()
    } else {
        RenderConfig::empty()
    }
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

fn file_style() -> Style {
    Style::new().blue()
}

/// Clear the current line to print arbitrary text after a prompt.
fn clear_line() {
    print!("\r");
}
