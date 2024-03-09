use crate::cli::CliConfig;
use crate::dialog::utils::{colorize_render_config, create_render_config};
use crate::dialog::PromptResult;
use cargo_wizard::PredefinedTemplateKind;
use clap::ValueEnum;
use inquire::ui::{Color, RenderConfig};
use inquire::Select;
use std::fmt::{Display, Formatter};

pub fn prompt_select_template(cli_config: &CliConfig) -> PromptResult<PredefinedTemplateKind> {
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
    let render_config = create_render_config(cli_config);
    colorize_render_config(cli_config, render_config, Color::DarkCyan)
}
