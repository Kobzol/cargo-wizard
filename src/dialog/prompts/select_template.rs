use crate::cli::CliConfig;
use crate::dialog::utils::{colorize_render_config, create_render_config};
use crate::dialog::{PromptResult, TemplateSource};
use cargo_wizard::PredefinedTemplateKind;
use clap::ValueEnum;
use inquire::Select;
use inquire::ui::{Color, RenderConfig};
use std::fmt::{Display, Formatter};

pub fn prompt_select_template(
    cli_config: &CliConfig,
    has_current_settings: bool,
) -> PromptResult<TemplateSource> {
    let options = has_current_settings
        .then_some(TemplateSource::CurrentProject)
        .into_iter()
        .chain(
            PredefinedTemplateKind::value_variants()
                .iter()
                .map(|template| TemplateSource::Predefined(*template)),
        )
        .collect();
    Ok(
        Select::new("Select the template that you want to apply:", options)
            .with_render_config(template_render_config(cli_config))
            .prompt()?,
    )
}

impl Display for TemplateSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TemplateSource::CurrentProject => {
                "Current project settings: edit the existing configuration"
            }
            TemplateSource::Predefined(PredefinedTemplateKind::FastCompile) => {
                "FastCompile: minimize compile times"
            }
            TemplateSource::Predefined(PredefinedTemplateKind::FastRuntime) => {
                "FastRuntime: maximize runtime performance"
            }
            TemplateSource::Predefined(PredefinedTemplateKind::MinSize) => {
                "MinSize: minimize binary size"
            }
        })
    }
}

fn template_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    let render_config = create_render_config(cli_config);
    colorize_render_config(cli_config, render_config, Color::DarkCyan)
}
