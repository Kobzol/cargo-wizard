use crate::cli::CliConfig;
use crate::dialog::utils::create_render_config;
use crate::dialog::PromptResult;
use cargo_wizard::CargoManifest;
use inquire::ui::{Color, RenderConfig};
use inquire::{min_length, Select, Text};
use std::fmt::{Display, Formatter};

pub fn prompt_select_profile(
    cli_config: &CliConfig,
    existing_profiles: Vec<String>,
) -> PromptResult<String> {
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
    let mut original_profiles: Vec<_> = existing_profiles
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
        Profile::CreateNew => prompt_enter_profile_name(cli_config)?,
    };

    Ok(profile)
}

fn prompt_enter_profile_name(cli_config: &CliConfig) -> PromptResult<String> {
    Ok(Text::new("Select profile name:")
        .with_validator(min_length!(1))
        .with_render_config(profile_render_config(cli_config))
        .prompt()?)
}

fn profile_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    let mut render_config = create_render_config(cli_config);
    render_config.answer = render_config.option.with_fg(Color::DarkGreen);
    render_config.selected_option = render_config
        .selected_option
        .map(|s| s.with_fg(Color::DarkGreen));
    render_config
}
