use anyhow::Context;

use cargo_wizard::{
    parse_workspace, resolve_manifest_path, BuiltinProfile, PredefinedTemplateKind, Profile,
    Template, TemplateItemId, TomlValue, WizardOptions,
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

pub use crate::dialog::known_options::KnownCargoOptions;
pub use utils::profile_from_str;

pub fn run_root_dialog(
    cli_config: CliConfig,
    cargo_options: KnownCargoOptions,
    options: WizardOptions,
) -> PromptResult<()> {
    let manifest_path = resolve_manifest_path().context("Cannot resolve Cargo.toml path")?;
    let workspace = parse_workspace(&manifest_path)?;
    let template_kind = prompt_select_template(&cli_config)?;

    let existing_profiles = workspace
        .existing_profiles()
        .iter()
        .filter_map(|s| profile_from_str(s).ok())
        .collect();
    let profile = prompt_select_profile(&cli_config, existing_profiles)?;

    let template = template_kind.build_template(&options);
    let template = prompt_customize_template(&cli_config, &cargo_options, template)?;

    let diff_result = prompt_confirm_diff(&cli_config, workspace, &profile, &template)?;
    match diff_result {
        ConfirmDiffPromptResponse::Accepted(workspace) => {
            workspace.write()?;
            on_template_applied(&cargo_options, template_kind, &template, &profile);
        }
        ConfirmDiffPromptResponse::Denied => {}
        ConfirmDiffPromptResponse::NoDiff => {
            println!("Nothing to apply, the profile already matched the template");
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
    if template.get_item(TemplateItemId::CodegenBackend)
        == Some(&TomlValue::String("cranelift".to_string()))
    {
        println!(
            "⚠️  Do not forget to install the cranelift component using `{}`.",
            utils::command_style().apply_to(
                "rustup component add rustc-codegen-cranelift-preview --toolchain nightly"
            )
        );
    }
    if let Some(linker) = template
        .get_item(TemplateItemId::Linker)
        .and_then(|v| v.to_toml_value().as_str().map(|s| s.to_string()))
    {
        println!(
            "⚠️  Do not forget to install the linker, e.g. using `{}`.",
            utils::command_style().apply_to(format!("sudo apt install {linker}"))
        );
    }

    if requires_nightly {
        println!("⚠️  You will have to use a nightly compiler.");
    }

    if let PredefinedTemplateKind::FastRuntime = template_kind {
        println!(
            "\nTip: Consider using the {} subcommand to further optimize your binary.",
            utils::command_style().apply_to("cargo-pgo")
        );
    }
}
