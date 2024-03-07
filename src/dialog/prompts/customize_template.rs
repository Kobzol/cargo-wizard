use std::fmt::{Display, Formatter};

use inquire::Select;

use cargo_wizard::{ProfileItemId, Template, TomlValue};

use crate::cli::CliConfig;
use crate::dialog::known_options::{KnownCargoOptions, PossibleValue};
use crate::dialog::utils::create_render_config;
use crate::dialog::PromptResult;

/// Customize the properties of a template, by choosing or modifying selected items.
pub fn prompt_customize_template(
    cli_config: &CliConfig,
    mut template: Template,
) -> PromptResult<Template> {
    loop {
        match prompt_choose_entry(cli_config, &template)? {
            ChooseEntryResponse::Confirm => {
                break;
            }
            ChooseEntryResponse::Modify(id) => {
                if let Some(value) = prompt_select_item_value(cli_config, &template, id)? {
                    match id {
                        ItemId::Profile(id) => {
                            template.profile.items.insert(id, value);
                        } // ItemId::Config(_id) => {
                          //     todo!();
                          // }
                    }
                }
            }
        }
    }
    Ok(template)
}

enum ChooseEntryResponse {
    Confirm,
    Modify(ItemId),
}

fn prompt_choose_entry(
    cli_config: &CliConfig,
    template: &Template,
) -> PromptResult<ChooseEntryResponse> {
    enum Row<'a> {
        Confirm,
        Profile {
            id: ProfileItemId,
            template: &'a Template,
        },
    }

    impl<'a> Display for Row<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Row::Confirm => f.write_str("<Confirm>"),
                Row::Profile { id, template } => {
                    write!(f, "{:<24}", ItemId::Profile(*id).to_string())?;

                    if let Some(value) = template.profile.items.get(id) {
                        let val = format!("[{}]", TOMLValueFormatter(&value));
                        write!(f, "{val:>10}")
                    } else {
                        f.write_str("         -")
                    }
                }
            }
        }
    }

    let rows = std::iter::once(Row::Confirm)
        .chain(
            KnownCargoOptions::get_profile_ids()
                .iter()
                .map(|&id| Row::Profile { id, template }),
        )
        .collect();
    let answer = Select::new("Select items to modify or confirm the template:", rows)
        .with_render_config(create_render_config(cli_config))
        .prompt()?;
    Ok(match answer {
        Row::Confirm => ChooseEntryResponse::Confirm,
        Row::Profile { id, .. } => ChooseEntryResponse::Modify(ItemId::Profile(id)),
    })
}

#[derive(Copy, Clone)]
enum ItemId {
    Profile(ProfileItemId),
    // Config(ConfigItemId),
}

impl ItemId {
    fn possible_values(&self) -> Vec<PossibleValue> {
        match self {
            ItemId::Profile(id) => KnownCargoOptions::get_profile_possible_values(*id),
        }
    }
}

impl Display for ItemId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            ItemId::Profile(id) => match id {
                ProfileItemId::DebugInfo => "Debug info",
                ProfileItemId::Strip => "Strip symbols",
                ProfileItemId::Lto => "Link-time optimizations",
                ProfileItemId::CodegenUnits => "Number of codegen units (CGUs)",
                ProfileItemId::Panic => "Panic handling mechanism",
                ProfileItemId::OptimizationLevel => "Optimization level",
            },
            // ItemId::Config(_id) => {
            //     todo!()
            // }
        };
        f.write_str(description)
    }
}

struct TOMLValueFormatter<'a>(&'a TomlValue);

impl<'a> Display for TOMLValueFormatter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            TomlValue::Int(value) => value.fmt(f),
            TomlValue::Bool(value) => value.fmt(f),
            TomlValue::String(value) => value.fmt(f),
        }
    }
}

fn prompt_select_item_value(
    cli_config: &CliConfig,
    template: &Template,
    item_id: ItemId,
) -> PromptResult<Option<TomlValue>> {
    enum Row {
        Value(PossibleValue),
        Cancel,
    }
    impl Display for Row {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Row::Value(value) => write!(
                    f,
                    "{}: {}",
                    value.value().to_toml_value().to_string(),
                    value.description()
                ),
                Row::Cancel => f.write_str("<Go back>"),
            }
        }
    }

    let mut rows: Vec<_> = item_id
        .possible_values()
        .into_iter()
        .map(Row::Value)
        .collect();
    rows.push(Row::Cancel);

    let existing_value = match item_id {
        ItemId::Profile(id) => template.profile.items.get(&id),
        // ItemId::Config(_id) => {
        //     todo!()
        // template..items.get(&entry.label)
        // }
    };
    // Select "Go back" as a default if no value is selected
    let index = existing_value
        .and_then(|value| {
            item_id
                .possible_values()
                .iter()
                .position(|v| v.value() == value)
        })
        .unwrap_or(item_id.possible_values().len());

    let selected = Select::new(&format!("Select value for `{}`:", item_id), rows)
        .with_starting_cursor(index)
        .with_help_message("↑↓ to move, enter to select, type to filter, ESC to cancel")
        .with_render_config(create_render_config(cli_config))
        .prompt_skippable()?;

    let result = match selected {
        Some(selected) => match selected {
            Row::Value(value) => Some(value.value().clone()),
            Row::Cancel => None,
        },
        None => None,
    };

    Ok(result)
}
