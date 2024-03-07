use std::fmt::{Display, Formatter};

use console::{style, Style};
use inquire::Confirm;
use similar::ChangeTag;

use cargo_wizard::{CargoWorkspace, ModificationResult, ModifiedWorkspace, Profile, Template};

use crate::cli::CliConfig;
use crate::dialog::utils::{clear_line, create_render_config, file_style};
use crate::dialog::PromptResult;

pub enum ConfirmDiffPromptResponse {
    Accepted(ModifiedWorkspace),
    Denied,
    NoDiff,
}

pub fn prompt_confirm_diff(
    cli_config: &CliConfig,
    workspace: CargoWorkspace,
    profile: &Profile,
    template: Template,
) -> PromptResult<ConfirmDiffPromptResponse> {
    let modified = workspace.apply_template(profile, template)?;

    // Cargo.toml
    let manifest_diff = match &modified.manifest {
        ModificationResult::NoChange => None,
        ModificationResult::Modified { old, new } => {
            Some(render_diff(&old.get_text(), &new.get_text()))
        }
    };
    let manifest_changed = manifest_diff.is_some();
    if let Some(diff) = manifest_diff {
        clear_line();
        println!("{}", file_style().apply_to("Cargo.toml"));
        println!("{diff}");
    }

    // .cargo/config.toml
    let config_diff = match &modified.config {
        ModificationResult::NoChange => None,
        ModificationResult::Modified { old, new } => {
            Some(render_diff(&old.get_text(), &new.get_text()))
        }
    };
    let config_changed = config_diff.is_some();
    if let Some(diff) = config_diff {
        clear_line();
        println!("{}", file_style().apply_to(".cargo/config.toml"));
        println!("{diff}");
    }

    if !manifest_changed && !config_changed {
        return Ok(ConfirmDiffPromptResponse::NoDiff);
    }

    let multiple_diffs = manifest_changed && config_changed;
    let answer = Confirm::new(&format!(
        "Do you want to apply the above diff{}?",
        if multiple_diffs { "s" } else { "" }
    ))
    .with_default(true)
    .with_render_config(create_render_config(cli_config))
    .prompt()?;

    Ok(match answer {
        true => ConfirmDiffPromptResponse::Accepted(modified),
        false => ConfirmDiffPromptResponse::Denied,
    })
}

// Taken from https://github.com/mitsuhiko/similar/blob/main/examples/terminal-inline.rs
fn render_diff(original: &str, new: &str) -> String {
    use std::fmt::Write;

    struct Line(Option<usize>);

    impl Display for Line {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            match self.0 {
                None => write!(f, "    "),
                Some(idx) => write!(f, "{:<4}", idx + 1),
            }
        }
    }

    let diff = similar::TextDiff::from_lines(original, new);
    let mut output = String::new();
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            write!(output, "{:-^1$}", "-", 80).unwrap();
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => ("|", Style::new().dim()),
                };
                write!(
                    output,
                    "{}{} {} ",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )
                .unwrap();
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(output, "{}", s.apply_to(value).underlined().on_black()).unwrap();
                    } else {
                        write!(output, "{}", s.apply_to(value)).unwrap();
                    }
                }
                if change.missing_newline() {
                    writeln!(output).unwrap();
                }
            }
        }
    }
    output
}
