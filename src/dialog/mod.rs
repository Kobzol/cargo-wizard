use anyhow::Context;

use cargo_wizard::{
    BuiltinProfile, PredefinedTemplateKind, Profile, Template, WizardOptions, parse_workspace,
    resolve_manifest_path,
};
pub use error::{DialogError, PromptResult};
pub use utils::profile_from_str;

use crate::cli::CliConfig;
pub use crate::dialog::known_options::KnownCargoOptions;
use crate::dialog::prompts::confirm_diff::{ConfirmDiffPromptResponse, prompt_confirm_diff};
use crate::dialog::prompts::customize_template::prompt_customize_template;
use crate::dialog::prompts::select_profile::prompt_select_profile;
use crate::dialog::prompts::select_template::prompt_select_template;

mod error;
mod known_options;
mod prompts;
mod utils;

pub fn run_root_dialog(
    cli_config: CliConfig,
    cargo_options: KnownCargoOptions,
    options: WizardOptions,
) -> PromptResult<()> {
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let workspace = parse_workspace(&manifest_path)?;

    let existing_profiles = workspace
        .existing_profiles()
        .iter()
        .filter_map(|s| profile_from_str(s).ok())
        .collect();
    let profile = prompt_select_profile(&cli_config, existing_profiles)?;

    let template_kind = prompt_select_template(&cli_config)?;
    let mut template = template_kind.build_template(&options);

    loop {
        template = prompt_customize_template(&cli_config, &cargo_options, template)?;

        let diff_result = prompt_confirm_diff(&cli_config, workspace.clone(), &profile, &template)?;
        match diff_result {
            ConfirmDiffPromptResponse::Accepted(workspace) => {
                workspace.write()?;
                on_template_applied(&cargo_options, template_kind, &template, &profile);
                break;
            }
            ConfirmDiffPromptResponse::Denied => {}
            ConfirmDiffPromptResponse::NoDiff => {
                println!("Nothing to apply, the profile already matched the template");
                break;
            }
        }
    }

    Ok(())
}

pub fn on_template_applied(
    options: &KnownCargoOptions,
    template_kind: PredefinedTemplateKind,
    template: &Template,
    profile: &Profile,
) {
    utils::clear_line();
    println!(
        "✅ Template {} applied to profile {}.",
        utils::template_style().apply_to(match template_kind {
            PredefinedTemplateKind::FastCompile => "FastCompile",
            PredefinedTemplateKind::FastRuntime => "FastRuntime",
            PredefinedTemplateKind::MinSize => "MinSize",
        }),
        utils::profile_style().apply_to(profile.name())
    );

    let requires_nightly = template
        .iter_items()
        .map(|(id, _)| id)
        .any(|id| options.get_metadata(id).requires_nightly());
    let profile_flag = match profile {
        Profile::Builtin(BuiltinProfile::Dev) => None,
        Profile::Builtin(BuiltinProfile::Release) => Some("--release".to_string()),
        Profile::Custom(profile) => Some(format!("--profile={profile}")),
    };
    if let Some(flag) = profile_flag {
        let channel = if requires_nightly { "+nightly " } else { "" };

        println!(
            "⚠️  Do not forget to run `{}` to use the selected profile.",
            utils::command_style().apply_to(format!("cargo {channel}<cmd> {flag}"))
        );
    }

    for (id, value) in template.iter_items() {
        if let Some(message) = options.get_metadata(id).on_applied(value) {
            println!("{message}");
        }
    }

    if requires_nightly {
        println!("⚠️  You will have to use a nightly compiler.");
    }

    match template_kind {
        PredefinedTemplateKind::FastCompile => {
            if !requires_nightly {
                println!(
                    "Tip: run `cargo-wizard` with the `{}` flag to discover nightly-only configuration options.",
                    utils::command_style().apply_to("--nightly")
                );
            }
        }
        PredefinedTemplateKind::FastRuntime => {
            println!(
                "Tip: consider using the {} subcommand to further optimize your binary.",
                utils::command_style().apply_to("cargo-pgo")
            );
        }
        PredefinedTemplateKind::MinSize => {}
    }
    let info_url = match template_kind {
        PredefinedTemplateKind::FastRuntime | PredefinedTemplateKind::FastCompile => {
            "https://nnethercote.github.io/perf-book/build-configuration.html"
        }
        PredefinedTemplateKind::MinSize => "https://github.com/johnthagen/min-sized-rust",
    };
    println!(
        "Tip: find more information at {}.",
        utils::command_style().apply_to(info_url)
    );
}
