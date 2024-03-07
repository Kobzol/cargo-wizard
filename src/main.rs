use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::Parser;

use cargo_wizard::{parse_workspace, resolve_manifest_path, PredefinedTemplateKind, Profile};

use crate::cli::CliConfig;
use crate::dialog::{on_template_applied, profile_from_str, run_root_dialog, DialogError};

mod cli;
mod dialog;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about)]
#[clap(bin_name("cargo"))]
#[clap(disable_help_subcommand(true))]
enum Args {
    #[clap(author, version, about)]
    Wizard(InnerArgs),
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ColorPolicy {
    Auto,
    Always,
    Never,
}

#[derive(clap::Parser, Debug)]
struct InnerArgs {
    /// Console color policy.
    #[arg(
        long,
        default_value_t = ColorPolicy::Auto,
        value_enum,
        global = true,
        help_heading("GLOBAL OPTIONS"),
        hide_short_help(true)
    )]
    colors: ColorPolicy,

    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Clone, Debug)]
struct ProfileArg(Profile);

impl FromStr for ProfileArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        profile_from_str(s).map(ProfileArg)
    }
}

#[derive(clap::Parser, Debug)]
struct ProfileArgs {
    /// Template that will be applied to the selected Cargo profile.
    template: PredefinedTemplateKind,
    /// Cargo profile that should be created or modified.
    profile: ProfileArg,
}

#[derive(clap::Parser, Debug)]
enum SubCommand {
    /// Apply a predefined template to the selected profile.
    Apply {
        #[clap(flatten)]
        args: ProfileArgs,
        /// Path to a Cargo.toml manifest.
        /// If not specified, it will be resolved to the current Cargo workspace.
        #[clap(long)]
        manifest_path: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Wizard(args) => {
            let cli_config = setup_cli(args.colors);
            match args.subcmd {
                Some(SubCommand::Apply {
                    args,
                    manifest_path,
                }) => {
                    let manifest_path = match manifest_path {
                        Some(path) => path,
                        None => {
                            resolve_manifest_path().context("Cannot resolve Cargo.toml path")?
                        }
                    };
                    let workspace = parse_workspace(&manifest_path)?;
                    let template = args.template.build_template();
                    let modified = workspace.apply_template(&args.profile.0, &template)?;
                    modified.write()?;
                    on_template_applied(args.template, &template, &args.profile.0);
                }
                None => {
                    if let Err(error) = run_root_dialog(cli_config) {
                        match error {
                            DialogError::Interrupted => {
                                // Print an empty line when the app is interrupted, to avoid
                                // overwriting the last line.
                                println!();
                            }
                            DialogError::Generic(error) => {
                                panic!("{error:?}");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn setup_cli(policy: ColorPolicy) -> CliConfig {
    let mut use_colors = match policy {
        ColorPolicy::Always => true,
        ColorPolicy::Auto => atty::is(atty::Stream::Stdout),
        ColorPolicy::Never => false,
    };

    if std::env::var("NO_COLOR") == Ok("1".to_string()) {
        use_colors = false;
    }

    console::set_colors_enabled(use_colors);
    console::set_colors_enabled_stderr(use_colors);
    CliConfig::new(use_colors)
}
