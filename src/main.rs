use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

use cargo_wizard::{parse_manifest, resolve_manifest_path};

use crate::cli::PredefinedTemplate;
use crate::dialog::dialog_root;

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

#[derive(clap::Parser, Debug)]
struct InnerArgs {
    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(clap::Parser, Debug)]
struct ProfileArgs {
    /// Cargo profile that should be created or modified.
    profile: String,
    /// Template that will be applied to the selected Cargo profile.
    template: PredefinedTemplate,
}

#[derive(clap::Parser, Debug)]
enum SubCommand {
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
        Args::Wizard(args) => match args.subcmd {
            Some(SubCommand::Apply {
                args,
                manifest_path,
            }) => {
                let manifest_path = match manifest_path {
                    Some(path) => path,
                    None => resolve_manifest_path().context("Cannot resolve Cargo.toml path")?,
                };
                let manifest = parse_manifest(&manifest_path)?;
                let template = args.template.resolve_to_template();
                let manifest = manifest.apply_profile(&args.profile, template)?;
                manifest.write(&manifest_path)?;
            }
            None => {
                dialog_root().context("Interactive dialog failed")?;
            }
        },
    }

    Ok(())
}
