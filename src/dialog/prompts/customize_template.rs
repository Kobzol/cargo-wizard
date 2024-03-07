use std::fmt::{Display, Formatter};
use std::str;
use std::str::FromStr;

use inquire::validator::{ErrorMessage, Validation};
use inquire::{CustomType, Select};

use cargo_wizard::{Template, TemplateItemId, TomlValue};

use crate::cli::CliConfig;
use crate::dialog::known_options::{
    KnownCargoOptions, PossibleValue, PossibleValueSet, SelectedPossibleValue, TomlValueKind,
};
use crate::dialog::utils::create_render_config;
use crate::dialog::PromptResult;

/// Customize the properties of a template, by choosing or modifying selected items.
pub fn prompt_customize_template(
    cli_config: &CliConfig,
    mut template: Template,
) -> PromptResult<Template> {
    loop {
        match prompt_choose_item_or_confirm_template(cli_config, &template)? {
            ChooseItemResponse::ConfirmTemplate => {
                break;
            }
            ChooseItemResponse::ModifyItem(id) => {
                match prompt_select_item_value(cli_config, &template, id)? {
                    SelectItemValueResponse::Set(value) => {
                        template.items.insert(id.0, value);
                    }
                    SelectItemValueResponse::Unset => {
                        template.items.shift_remove(&id.0);
                    }
                    SelectItemValueResponse::Cancel => {}
                }
            }
        }
    }
    Ok(template)
}

enum ChooseItemResponse {
    ConfirmTemplate,
    ModifyItem(ItemId),
}

/// Choose a profile/config item that should be modified,
/// or confirm the template.
fn prompt_choose_item_or_confirm_template(
    cli_config: &CliConfig,
    template: &Template,
) -> PromptResult<ChooseItemResponse> {
    enum Row<'a> {
        Confirm,
        Item { id: ItemId, template: &'a Template },
    }

    impl<'a> Display for Row<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Row::Confirm => f.write_str("<Confirm>"),
                Row::Item { id, template } => {
                    write!(f, "{:<30}", id.to_string())?;

                    if let Some(value) = template.items.get(&id.0) {
                        let val = format!("[{}]", TomlValueDisplay(&value));
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
            KnownCargoOptions::get_all_ids()
                .iter()
                .map(|&id| Row::Item {
                    id: ItemId(id),
                    template,
                }),
        )
        .collect();
    let answer = Select::new("Select items to modify or confirm the template:", rows)
        .with_render_config(create_render_config(cli_config))
        .prompt()?;
    Ok(match answer {
        Row::Confirm => ChooseItemResponse::ConfirmTemplate,
        Row::Item { id, .. } => ChooseItemResponse::ModifyItem(id),
    })
}

#[derive(Copy, Clone)]
struct ItemId(TemplateItemId);

impl ItemId {
    fn value_set(&self) -> PossibleValueSet {
        KnownCargoOptions::get_possible_values(self.0)
    }

    fn selected_value(&self, template: &Template) -> Option<TomlValue> {
        template.items.get(&self.0).cloned()
    }
}

impl Display for ItemId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self.0 {
            TemplateItemId::DebugInfo => "Debug info",
            TemplateItemId::Strip => "Strip symbols",
            TemplateItemId::Lto => "Link-time optimizations",
            TemplateItemId::CodegenUnits => "Number of codegen units (CGUs)",
            TemplateItemId::Panic => "Panic handling mechanism",
            TemplateItemId::OptimizationLevel => "Optimization level",
            TemplateItemId::TargetCpuInstructionSet => "Target CPU instruction set",
        };
        f.write_str(description)
    }
}

struct TomlValueDisplay<'a>(&'a TomlValue);

impl<'a> Display for TomlValueDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            TomlValue::Int(value) => value.fmt(f),
            TomlValue::Bool(value) => value.fmt(f),
            TomlValue::String(value) => value.fmt(f),
        }
    }
}

struct ValueKindDisplay(TomlValueKind);

impl Display for ValueKindDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self.0 {
            TomlValueKind::Int => "int",
            TomlValueKind::String => "string",
        };
        f.write_str(kind)
    }
}

enum SelectItemValueResponse {
    Set(TomlValue),
    Unset,
    Cancel,
}

/// Select a value for a specific profile or config item.
/// This function is passed a template so that it knows if any existing value
/// is already selected.
fn prompt_select_item_value(
    cli_config: &CliConfig,
    template: &Template,
    item_id: ItemId,
) -> PromptResult<SelectItemValueResponse> {
    enum Row<'a> {
        ConstantValue(PossibleValue),
        CustomValue {
            kind: TomlValueKind,
            selected_value: &'a SelectedPossibleValue,
        },
        Unset,
        Cancel,
    }
    impl<'a> Display for Row<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Row::ConstantValue(value) => write!(
                    f,
                    "{:<40} {}",
                    value.description(),
                    value.value().to_toml_value().to_string(),
                ),
                Row::CustomValue {
                    kind,
                    selected_value,
                } => {
                    write!(f, "{:<40}", "Custom value")?;
                    match selected_value {
                        SelectedPossibleValue::Custom { value } => {
                            write!(f, " {}", TomlValueDisplay(value))
                        }
                        _ => write!(f, "({})", ValueKindDisplay(*kind)),
                    }
                }
                Row::Unset => f.write_str("<Unset value>"),
                Row::Cancel => f.write_str("<Go back>"),
            }
        }
    }

    let value_set = item_id.value_set();
    let selected_value = item_id.selected_value(template);
    let selected_value = selected_value
        .clone()
        .map(|v| value_set.get_selected_value(v))
        .unwrap_or(SelectedPossibleValue::None);

    let mut rows: Vec<_> = value_set
        .get_possible_values()
        .into_iter()
        .cloned()
        .map(Row::ConstantValue)
        .collect();
    if let Some(kind) = value_set.get_custom_value_kind() {
        rows.push(Row::CustomValue {
            kind,
            selected_value: &selected_value,
        });
    }

    rows.push(Row::Unset);
    rows.push(Row::Cancel);

    let mut default_index = value_set.get_possible_values().len();
    if value_set.get_custom_value_kind().is_some() {
        default_index += 1;
    }

    let index = match selected_value {
        SelectedPossibleValue::Constant { index, .. } => index,
        // Select "Custom value" as a default if a custom value is selected
        SelectedPossibleValue::Custom { .. } => value_set.get_possible_values().len(),
        // Select "Go back" as a default if no value is selected
        SelectedPossibleValue::None => default_index,
    };

    let selected = Select::new(&format!("Select value for `{}`:", item_id), rows)
        .with_starting_cursor(index)
        .with_help_message("↑↓ to move, enter to select, type to filter, ESC to cancel")
        .with_render_config(create_render_config(cli_config))
        .prompt_skippable()?;

    let result = match selected {
        Some(selected) => match selected {
            Row::ConstantValue(value) => SelectItemValueResponse::Set(value.value().clone()),
            Row::CustomValue { kind, .. } => {
                let value = prompt_enter_custom_value(cli_config, kind)?;
                SelectItemValueResponse::Set(value)
            }
            Row::Unset => SelectItemValueResponse::Unset,
            Row::Cancel => SelectItemValueResponse::Cancel,
        },
        None => SelectItemValueResponse::Cancel,
    };

    Ok(result)
}

/// Enter a custom TOML value of the given kind.
fn prompt_enter_custom_value(
    cli_config: &CliConfig,
    kind: TomlValueKind,
) -> PromptResult<TomlValue> {
    #[derive(Clone)]
    struct Value(TomlValue);

    impl Display for Value {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            TomlValueDisplay(&self.0).fmt(f)
        }
    }

    impl FromStr for Value {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.is_empty() {
                return Err("Value must not be empty");
            }

            if let Ok(value) = bool::from_str(s) {
                Ok(Self(TomlValue::Bool(value)))
            } else if let Ok(value) = i64::from_str(s) {
                Ok(Self(TomlValue::Int(value)))
            } else {
                Ok(Self(TomlValue::String(String::from(s))))
            }
        }
    }

    let value = CustomType::<Value>::new(&format!(
        "Enter custom value of type {}: ",
        ValueKindDisplay(kind)
    ))
    .with_validator(move |val: &Value| match (kind, &val.0) {
        (TomlValueKind::Int, TomlValue::Int(_)) => Ok(Validation::Valid),
        (kind, value) => Ok(Validation::Invalid(ErrorMessage::Custom(format!(
            "Invalid TOML type, expected `{}`, got {}",
            ValueKindDisplay(kind),
            match value {
                TomlValue::Int(_) => "int",
                TomlValue::Bool(_) => "bool",
                TomlValue::String(_) => "string",
            }
        )))),
    })
    .with_render_config(create_render_config(cli_config))
    .prompt()?;

    Ok(value.0)
}
