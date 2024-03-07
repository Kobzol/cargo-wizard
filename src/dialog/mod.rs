use anyhow::Context;

use cargo_wizard::{
    parse_workspace, resolve_manifest_path, BuiltinProfile, PredefinedTemplateKind, Profile,
};
pub use error::{DialogError, PromptResult};

use crate::cli::CliConfig;
use crate::dialog::prompts::confirm_diff::{prompt_confirm_diff, ConfirmDiffPromptResponse};
use crate::dialog::prompts::customize_template::prompt_customize_template;
use crate::dialog::prompts::select_profile::prompt_select_profile;
use crate::dialog::prompts::select_template::prompt_select_template;

mod error;
mod known_options;
mod prompts;
mod utils;

pub use utils::profile_from_str;

pub fn run_root_dialog(cli_config: CliConfig) -> PromptResult<()> {
    let template_kind = prompt_select_template(&cli_config)?;
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let workspace = parse_workspace(&manifest_path)?;

    let existing_profiles = workspace
        .existing_profiles()
        .iter()
        .filter_map(|s| profile_from_str(s).ok())
        .collect();
    let profile = prompt_select_profile(&cli_config, existing_profiles)?;

    let template = template_kind.build_template();
    let template = prompt_customize_template(&cli_config, template)?;

    let diff_result = prompt_confirm_diff(&cli_config, workspace, &profile, template)?;
    match diff_result {
        ConfirmDiffPromptResponse::Accepted(workspace) => {
            workspace.write()?;
            on_template_applied(template_kind, &profile);
        }
        ConfirmDiffPromptResponse::Denied => {}
        ConfirmDiffPromptResponse::NoDiff => {
            println!("Nothing to apply, the profile already matched the template");
        }
    }

    Ok(())
}

pub fn on_template_applied(template: PredefinedTemplateKind, profile: &Profile) {
    utils::clear_line();
    println!(
        "✅ Template {} applied to profile {}.",
        utils::template_style().apply_to(match template {
            PredefinedTemplateKind::FastCompile => "FastCompile",
            PredefinedTemplateKind::FastRuntime => "FastRuntime",
            PredefinedTemplateKind::MinSize => "MinSize",
        }),
        utils::profile_style().apply_to(profile.name())
    );

    let profile_flag = match profile {
        Profile::Builtin(BuiltinProfile::Dev) => None,
        Profile::Builtin(BuiltinProfile::Release) => Some("--release".to_string()),
        Profile::Custom(profile) => Some(format!("--profile={profile}")),
    };
    if let Some(flag) = profile_flag {
        println!(
            "❗ Do not forget to run `{}` to use the selected profile.",
            utils::command_style().apply_to(format!("cargo <cmd> {flag}"))
        );
    }

    if let PredefinedTemplateKind::FastRuntime = template {
        println!(
            "\nTip: Consider using the {} subcommand to further optimize your binary.",
            utils::command_style().apply_to("cargo-pgo")
        );
    }
}
