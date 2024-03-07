use crate::cli::CliConfig;
use crate::dialog::utils::create_render_config;
use crate::dialog::PromptResult;
use cargo_wizard::{
    CargoOption, ConfigItemId, KnownCargoOptions, PossibleValue, ProfileItemId, Template, TomlValue,
};
use inquire::Select;
use std::fmt::{Display, Formatter};

/// Customize the properties of a template, by choosing or modifying selected items.
pub fn prompt_customize_template(
    cli_config: &CliConfig,
    known_options: KnownCargoOptions,
    mut template: Template,
) -> PromptResult<Template> {
    loop {
        match prompt_choose_entry(cli_config, &known_options, &template)? {
            ChooseEntryResponse::Confirm => {
                break;
            }
            ChooseEntryResponse::Modify(option) => {
                if let Some(value) = prompt_select_item_value(cli_config, &template, &option)? {
                    match option.0.id() {
                        ItemId::Profile(id) => {
                            template.profile.items.insert(id, value);
                        }
                        ItemId::Config(id) => {}
                    }
                }
            }
        }
    }
    Ok(template)
}

enum ChooseEntryResponse {
    Confirm,
    Modify(AnyItem),
}

fn prompt_choose_entry(
    cli_config: &CliConfig,
    known_options: &KnownCargoOptions,
    template: &Template,
) -> PromptResult<ChooseEntryResponse> {
    enum Row<'a> {
        Confirm,
        Profile {
            option: &'a CargoOption<ProfileItemId>,
            template: &'a Template,
        },
    }

    impl<'a> Display for Row<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Row::Confirm => f.write_str("<Confirm>"),
                Row::Profile { template, option } => {
                    write!(f, "{:<24}", ProfileIdDisplay(option.id()).to_string())?;

                    if let Some(value) = template.profile.items.get(&option.id()) {
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
            known_options
                .get_profile()
                .iter()
                .map(|option| Row::Profile { option, template }),
        )
        .collect();
    let answer = Select::new("Select items to modify or confirm the template:", rows)
        .with_render_config(create_render_config(cli_config))
        .prompt()?;
    Ok(match answer {
        Row::Confirm => ChooseEntryResponse::Confirm,
        Row::Profile { option, .. } => {
            ChooseEntryResponse::Modify(AnyItem(option.with_id(ItemId::Profile(option.id()))))
        }
    })
}

#[derive(Copy, Clone)]
enum ItemId {
    Profile(ProfileItemId),
    Config(ConfigItemId),
}

impl Display for ItemId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            ItemId::Profile(id) => match id {
                ProfileItemId::DebugInfo => "Debug info",
                ProfileItemId::Strip => "Strip symbols",
                ProfileItemId::Lto => "Link-time optimizations",
            },
            ItemId::Config(_id) => {
                todo!()
            }
        };
        f.write_str(description)
    }
}

struct AnyItem(CargoOption<ItemId>);

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

struct ProfileIdDisplay(ProfileItemId);

impl Display for ProfileIdDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self.0 {
            ProfileItemId::DebugInfo => "Debug info",
            ProfileItemId::Strip => "Strip symbols",
            ProfileItemId::Lto => "Link-time optimizations",
        };
        f.write_str(description)
    }
}

fn prompt_select_item_value(
    cli_config: &CliConfig,
    template: &Template,
    item: &AnyItem,
) -> PromptResult<Option<TomlValue>> {
    enum Row<'a> {
        Value(&'a PossibleValue),
        Cancel,
    }
    impl<'a> Display for Row<'a> {
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

    let mut rows: Vec<_> = item.0.possible_values().iter().map(Row::Value).collect();
    rows.push(Row::Cancel);

    let existing_value = match item.0.id() {
        ItemId::Profile(id) => template.profile.items.get(&id),
        ItemId::Config(_id) => {
            todo!()
            // template..items.get(&entry.label)
        }
    };
    // Select "Go back" as a default if no value is selected
    let index = existing_value
        .and_then(|value| {
            item.0
                .possible_values()
                .iter()
                .position(|v| v.value() == value)
        })
        .unwrap_or(item.0.possible_values().len());

    let selected = Select::new(&format!("Select value for `{}`:", item.0.id()), rows)
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
