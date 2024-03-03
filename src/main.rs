use std::path::PathBuf;

use clap::Parser;

use cargo_wizard::{fast_compile_template, parse_manifest, TomlProfileTemplate};

#[derive(clap::Parser)]
struct Args {
    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum PredefinedProfile {
    FastCompile,
}

impl PredefinedProfile {
    fn resolve_to_template(&self) -> TomlProfileTemplate {
        match self {
            PredefinedProfile::FastCompile => fast_compile_template(),
        }
    }
}

#[derive(clap::Parser, Debug)]
struct ProfileArgs {
    /// Cargo profile that should be created or modified.
    profile: String,
    /// Template that will be applied to the selected Cargo profile.
    template: PredefinedProfile,
}

#[derive(clap::Parser, Debug)]
enum SubCommand {
    Set {
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
    match args.subcmd {
        None => {}
        Some(SubCommand::Set {
            args,
            manifest_path,
        }) => {
            let manifest_path = manifest_path.unwrap();
            let manifest = parse_manifest(&manifest_path)?;
            let template = args.template.resolve_to_template();
            let manifest = manifest.apply_profile(&args.profile, template)?;
            manifest.write(&manifest_path)?;
        }
    }

    Ok(())
}
