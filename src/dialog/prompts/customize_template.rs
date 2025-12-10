use std::fmt::{Display, Formatter};
use std::str;
use std::str::FromStr;

use inquire::autocompletion::Replacement;
use inquire::ui::{Color, RenderConfig};
use inquire::validator::{ErrorMessage, Validation};
use inquire::{Autocomplete, CustomUserError, Select, Text};

use cargo_wizard::{Template, TemplateItemId, TomlValue};

use crate::cli::CliConfig;
use crate::dialog::PromptResult;
use crate::dialog::known_options::{
    CustomPossibleValue, KnownCargoOptions, PossibleValue, SelectedPossibleValue,
    TemplateItemMedata, TomlValueKind,
};
use crate::dialog::utils::{colorize_render_config, create_render_config};

/// Customize the properties of a template, by choosing or modifying selected items.
pub fn prompt_customize_template(
    cli_config: &CliConfig,
    options: &KnownCargoOptions,
    mut template: Template,
) -> PromptResult<Template> {
    loop {
        match prompt_choose_item_or_confirm_template(cli_config, options, &template)? {
            ChooseItemResponse::ConfirmTemplate => {
                break;
            }
            ChooseItemResponse::ModifyItem(id) => {
                match prompt_select_value_for_item(cli_config, options, &template, id)? {
                    SelectItemValueResponse::Set(value) => {
                        template.insert_item(id.0, value);
                    }
                    SelectItemValueResponse::Unset => {
                        template.remove_item(id.0);
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
    options: &KnownCargoOptions,
    template: &Template,
) -> PromptResult<ChooseItemResponse> {
    enum Row<'a> {
        Confirm,
        Item {
            id: ItemId,
            metadata: TemplateItemMedata,
            template: &'a Template,
        },
    }

    impl<'a> Display for Row<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Row::Confirm => f.write_str("<Confirm>"),
                Row::Item {
                    id,
                    metadata,
                    template,
                } => {
                    let mut notes = vec![];
                    if metadata.requires_nightly() {
                        notes.push("*");
                    }
                    if metadata.requires_unix() {
                        notes.push("^");
                    }
                    let name = format!(
                        "{id}{}",
                        if notes.is_empty() {
                            "".to_string()
                        } else {
                            format!(" {}", notes.join(""))
                        }
                    );
                    write!(f, "{name:<30}")?;

                    if let Some(value) = template.get_item(id.0) {
                        let val = format!("[{}]", TomlValueDisplay(value));
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
                    metadata: options.get_metadata(id),
                    template,
                }),
        )
        .collect();
    let answer = Select::new("Select items to modify or confirm the template:", rows)
        .with_page_size(12)
        .with_help_message(
            "↑↓ to move, enter to select, type to filter. * Requires nightly compiler ^ Requires Unix",
        )
        .with_render_config(customize_render_config(cli_config))
        .prompt()?;
    Ok(match answer {
        Row::Confirm => ChooseItemResponse::ConfirmTemplate,
        Row::Item { id, .. } => ChooseItemResponse::ModifyItem(id),
    })
}

#[derive(Copy, Clone)]
struct ItemId(TemplateItemId);

impl ItemId {
    fn value_set(&self, options: &KnownCargoOptions) -> TemplateItemMedata {
        options.get_metadata(self.0)
    }

    fn selected_value(&self, template: &Template) -> Option<TomlValue> {
        template.get_item(self.0).cloned()
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
            TemplateItemId::CodegenBackend => "Codegen backend",
            TemplateItemId::TargetCpuInstructionSet => "Target CPU instruction set",
            TemplateItemId::FrontendThreads => "Number of frontend threads",
            TemplateItemId::Linker => "Linker",
            TemplateItemId::Incremental => "Incremental compilation",
            TemplateItemId::SplitDebugInfo => "Split debug info",
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
fn prompt_select_value_for_item(
    cli_config: &CliConfig,
    options: &KnownCargoOptions,
    template: &Template,
    item_id: ItemId,
) -> PromptResult<SelectItemValueResponse> {
    enum Row<'a> {
        ConstantValue(PossibleValue),
        CustomValue {
            custom_value: &'a CustomPossibleValue,
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
                    value.value().to_toml_value(),
                ),
                Row::CustomValue {
                    custom_value,
                    selected_value,
                } => {
                    write!(f, "{:<40}", "Custom value")?;
                    match selected_value {
                        SelectedPossibleValue::Custom { value } => {
                            write!(f, " {}", TomlValueDisplay(value))
                        }
                        _ => write!(f, "({})", ValueKindDisplay(custom_value.kind())),
                    }
                }
                Row::Unset => f.write_str("<Unset value>"),
                Row::Cancel => f.write_str("<Go back>"),
            }
        }
    }

    let value_set = item_id.value_set(options);
    let selected_value = item_id.selected_value(template);
    let selected_value = selected_value
        .clone()
        .map(|v| value_set.get_selected_value(v))
        .unwrap_or(SelectedPossibleValue::None);

    let mut rows: Vec<_> = value_set
        .get_possible_values()
        .iter()
        .cloned()
        .map(Row::ConstantValue)
        .collect();
    if let Some(custom_value) = value_set.get_custom_value() {
        rows.push(Row::CustomValue {
            custom_value,
            selected_value: &selected_value,
        });
    }

    if item_id.selected_value(template).is_some() {
        rows.push(Row::Unset);
    }
    rows.push(Row::Cancel);

    let index = match selected_value {
        SelectedPossibleValue::Constant { index, .. } => index,
        // Select "Custom value" as a default if a custom value is selected
        SelectedPossibleValue::Custom { .. } => value_set.get_possible_values().len(),
        // Select "Go back" as a default if no value is selected
        SelectedPossibleValue::None => rows.len() - 1,
    };

    let selected = Select::new(&format!("Select value for `{}`:", item_id), rows)
        .with_starting_cursor(index)
        .with_help_message("↑↓ to move, enter to select, type to filter, ESC to cancel")
        .with_render_config(customize_render_config(cli_config))
        .prompt_skippable()?;

    let result = match selected {
        Some(selected) => match selected {
            Row::ConstantValue(value) => SelectItemValueResponse::Set(value.value().clone()),
            Row::CustomValue { custom_value, .. } => {
                let value = prompt_enter_custom_value(cli_config, custom_value)?;
                SelectItemValueResponse::Set(value)
            }
            Row::Unset { .. } => SelectItemValueResponse::Unset,
            Row::Cancel => SelectItemValueResponse::Cancel,
        },
        None => SelectItemValueResponse::Cancel,
    };

    Ok(result)
}

/// Enter a custom TOML value of the given kind.
fn prompt_enter_custom_value(
    cli_config: &CliConfig,
    custom_value: &CustomPossibleValue,
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

    #[derive(Clone)]
    struct AutoCompleter(Vec<String>);

    impl Autocomplete for AutoCompleter {
        fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
            Ok(self
                .0
                .clone()
                .into_iter()
                .filter(|v| v.contains(input))
                .collect())
        }

        fn get_completion(
            &mut self,
            _input: &str,
            highlighted_suggestion: Option<String>,
        ) -> Result<Replacement, CustomUserError> {
            Ok(highlighted_suggestion)
        }
    }

    // Ideally, we would use the CustomValue prompt here, but that doesn't support autocompletion.
    let kind = custom_value.kind();
    let value = Text::new(&format!(
        "Enter custom value of type {}: ",
        ValueKindDisplay(custom_value.kind())
    ))
    .with_autocomplete(AutoCompleter(custom_value.possible_entries().to_vec()))
    .with_validator(move |val: &str| {
        let val = match Value::from_str(val) {
            Ok(val) => val,
            Err(error) => return Ok(Validation::Invalid(ErrorMessage::Custom(error.to_string()))),
        };
        match kind {
            TomlValueKind::Int if matches!(val.0, TomlValue::Int(_)) => Ok(Validation::Valid),
            TomlValueKind::String if matches!(val.0, TomlValue::String(_)) => Ok(Validation::Valid),
            TomlValueKind::Int | TomlValueKind::String => {
                Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                    "Invalid TOML type, expected `{}`, got {}",
                    ValueKindDisplay(kind),
                    match val.0 {
                        TomlValue::Int(_) => "int",
                        TomlValue::Bool(_) => "bool",
                        TomlValue::String(_) => "string",
                    }
                ))))
            }
        }
    })
    .with_render_config(customize_render_config(cli_config))
    .with_help_message("↑↓ to select hint, tab to autocomplete hint, enter to submit")
    .prompt()?;

    // We expect that the value has been parsed successfully thanks to the validator above
    let value = Value::from_str(&value).expect("Could not parse value");
    Ok(value.0)
}

fn customize_render_config(cli_config: &CliConfig) -> RenderConfig<'static> {
    let render_config = create_render_config(cli_config);
    colorize_render_config(cli_config, render_config, Color::LightCyan)
}
