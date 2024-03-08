use crate::cli::CliConfig;
use crate::dialog::utils::create_render_config;
use crate::dialog::PromptResult;
use cargo_wizard::Profile;
use inquire::ui::{Color, RenderConfig};
use inquire::{min_length, Select, Text};
use std::fmt::{Display, Formatter};

pub fn prompt_select_profile(
    cli_config: &CliConfig,
    existing_profiles: Vec<Profile>,
) -> PromptResult<Profile> {
    enum ProfileRow {
        Dev,
        Release,
        Custom(String),
        CreateNew,
    }
    impl Display for ProfileRow {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                ProfileRow::Dev => f.write_str("dev (builtin)"),
                ProfileRow::Release => f.write_str("release (builtin)"),
                ProfileRow::Custom(name) => f.write_str(name),
                ProfileRow::CreateNew => f.write_str("<Create a new profile>"),
            }
        }
    }

    let mut profiles = vec![ProfileRow::Dev, ProfileRow::Release];
    let mut original_profiles: Vec<_> = existing_profiles
        .into_iter()
        .filter_map(|p| match p {
            Profile::Builtin(_) => None,
            Profile::Custom(custom) => Some(custom.clone()),
        })
        .collect();
    original_profiles.sort();
    profiles.extend(original_profiles.into_iter().map(ProfileRow::Custom));
    profiles.push(ProfileRow::CreateNew);

    let selected = Select::new(
        "Select the profile that you want to update/create:",
        profiles,
    )
    .with_render_config(profile_render_config(cli_config))
    .prompt()?;

    let profile = match selected {
        ProfileRow::Dev => Profile::dev(),
        ProfileRow::Release => Profile::release(),
        ProfileRow::Custom(name) => Profile::Custom(name),
        ProfileRow::CreateNew => prompt_enter_profile_name(cli_config)?,
    };

    Ok(profile)
}

fn prompt_enter_profile_name(cli_config: &CliConfig) -> PromptResult<Profile> {
    let profile = Text::new("Select profile name:")
        .with_validator(min_length!(1))
        .with_render_config(profile_render_config(cli_config))
        .prompt()?;
    let profile = match profile.as_str() {
        "dev" => Profile::dev(),
        "release" => Profile::release(),
        _ => Profile::Custom(profile),
    };
    Ok(profile)
}

fn profile_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    let mut render_config = create_render_config(cli_config);
    render_config.answer = render_config.option.with_fg(Color::DarkGreen);
    render_config.selected_option = render_config
        .selected_option
        .map(|s| s.with_fg(Color::DarkGreen));
    if cli_config.colors_enabled() {
        render_config.highlighted_option_prefix.style = render_config
            .highlighted_option_prefix
            .style
            .with_fg(Color::DarkGreen);
    }
    render_config
}
