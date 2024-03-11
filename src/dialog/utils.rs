use std::path::PathBuf;

use console::Style;
use inquire::ui::RenderConfig;

use cargo_wizard::Profile;

use crate::cli::CliConfig;

pub fn create_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    if cli_config.colors_enabled() {
        RenderConfig::default_colored()
    } else {
        RenderConfig::empty()
    }
}

pub fn colorize_render_config<'a>(
    cli_config: &CliConfig,
    mut config: RenderConfig<'a>,
    color: inquire::ui::Color,
) -> RenderConfig<'a> {
    if cli_config.colors_enabled() {
        config.answer = config.option.with_fg(color);
        config.selected_option = config.selected_option.map(|s| s.with_fg(color));
        config.highlighted_option_prefix.style =
            config.highlighted_option_prefix.style.with_fg(color);
        config.prompt_prefix.style = config.prompt_prefix.style.with_fg(color);
        config.answered_prompt_prefix.style = config.answered_prompt_prefix.style.with_fg(color);
    }
    config
}

pub fn template_style() -> Style {
    Style::new().cyan()
}

pub fn profile_style() -> Style {
    Style::new().green()
}

pub fn command_style() -> Style {
    Style::new().yellow()
}

pub fn file_style() -> Style {
    Style::new().blue()
}

/// Clear the current line to print arbitrary text after a prompt.
pub fn clear_line() {
    print!("\r");
}

pub fn profile_from_str(text: &str) -> Result<Profile, String> {
    if text.is_empty() {
        return Err("Profile cannot be empty".to_string());
    }
    let profile = match text {
        "dev" => Profile::dev(),
        "release" => Profile::release(),
        custom => Profile::Custom(custom.to_string()),
    };
    Ok(profile)
}

/// Quick check if a program with the given name can be found.
pub fn find_program_path(name: &str) -> Option<PathBuf> {
    which::which(name).ok()
}
