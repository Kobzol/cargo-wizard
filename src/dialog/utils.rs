use crate::cli::CliConfig;
use console::Style;
use inquire::ui::RenderConfig;

pub fn create_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    if cli_config.colors_enabled() {
        RenderConfig::default_colored()
    } else {
        RenderConfig::empty()
    }
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
