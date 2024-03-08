use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::Parser;
use rustc_version::Channel;

use cargo_wizard::{
    parse_workspace, resolve_manifest_path, PredefinedTemplateKind, Profile, WizardOptions,
};

use crate::cli::CliConfig;
use crate::dialog::{
    on_template_applied, profile_from_str, run_root_dialog, DialogError, KnownCargoOptions,
};

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
enum ColorPolicy {
    /// Use colors if the stdout is detected to be a terminal.
    Auto,
    /// Use colors.
    Always,
    /// Do not use colors.
    Never,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum NightlyOptions {
    /// Include nightly options if you invoke `cargo wizard` with a nightly compiler.
    Auto,
    /// Include nightly options.
    On,
    /// Do not include nightly options.
    Off,
}

#[derive(clap::Parser, Debug)]
struct InnerArgs {
    /// Console color policy.
    #[arg(
        long,
        value_enum,
        default_value_t = ColorPolicy::Auto,
        global = true,
        help_heading("GLOBAL OPTIONS"),
        hide_short_help(true)
    )]
    colors: ColorPolicy,

    /// Include profile configuration that requires a nightly compiler.
    #[arg(
        long,
        value_enum,
        default_value_t = NightlyOptions::Auto,
        default_missing_value = "on",
        global = true,
        num_args = 0..=1,
        require_equals = true,
        help_heading("GLOBAL OPTIONS"),
        hide_short_help(true)
    )]
    nightly: NightlyOptions,

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
struct ApplyArgs {
    /// Cargo profile that should be created or modified.
    profile: ProfileArg,
    /// Template that will be applied to the selected Cargo profile.
    template: PredefinedTemplateKind,
}

#[derive(clap::Parser, Debug)]
enum SubCommand {
    /// Apply a predefined template to the selected profile.
    Apply {
        #[clap(flatten)]
        args: ApplyArgs,
        /// Path to a Cargo.toml manifest.
        /// If not specified, it will be resolved to the current Cargo workspace.
        #[clap(long)]
        manifest_path: Option<PathBuf>,
    },
}

fn options_from_args(args: &InnerArgs) -> WizardOptions {
    let mut options = WizardOptions::default();
    let is_nightly = match args.nightly {
        NightlyOptions::Auto => {
            match rustc_version::version_meta() {
                Ok(metadata) => {
                    matches!(metadata.channel, Channel::Nightly)
                }
                Err(error) => {
                    eprintln!("Cannot get compiler channel, defaulting to *no* nightly options ({error:?}");
                    false
                }
            }
        }
        NightlyOptions::On => true,
        NightlyOptions::Off => false,
    };
    if is_nightly {
        options = options.with_nightly_items();
    }
    options
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Wizard(root_args) => {
            let options = options_from_args(&root_args);
            let cargo_options =
                KnownCargoOptions::create().context("Cannot get known Cargo options")?;
            let cli_config = setup_cli(root_args.colors);
            match root_args.subcmd {
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
                    let template = args.template.build_template(&options);
                    let modified = workspace.apply_template(&args.profile.0, &template)?;
                    modified.write()?;
                    on_template_applied(&cargo_options, args.template, &template, &args.profile.0);
                }
                None => {
                    if let Err(error) = run_root_dialog(cli_config, cargo_options, options) {
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
