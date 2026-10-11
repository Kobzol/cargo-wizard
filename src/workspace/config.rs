use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{Template, TemplateItemId, TomlValue};
use anyhow::Context;
use toml_edit::{Array, DocumentMut, Formatted, Value, table, value};

/// Config stored in `.cargo/config.toml` file.
#[derive(Debug, Clone)]
pub struct CargoConfig {
    path: PathBuf,
    document: DocumentMut,
}

impl CargoConfig {
    pub fn empty_from_manifest(manifest_path: &Path) -> Self {
        Self {
            path: config_path_from_manifest_path(manifest_path),
            document: Default::default(),
        }
    }

    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let config = std::fs::read_to_string(path).context("Cannot read config.toml file")?;
        let document = config
            .parse::<DocumentMut>()
            .context("Cannot parse config.toml file")?;

        Ok(Self {
            document,
            path: path.to_path_buf(),
        })
    }

    pub fn get_text(&self) -> String {
        self.document.to_string()
    }

    pub fn apply_template(mut self, template: &Template) -> anyhow::Result<Self> {
        let build_values: anyhow::Result<Vec<_>> = template
            .iter_items()
            .filter_map(|(id, value)| {
                let descriptor = config_item_descriptor(id)?;
                let ConfigItemEncoding::BuildString { key } = descriptor.encoding else {
                    return None;
                };
                Some(match value {
                    TomlValue::String(value) => Ok((key, value.clone())),
                    _ => Err(anyhow::anyhow!(
                        "build.{key} in the template must be a string"
                    )),
                })
            })
            .collect();
        let build_values = build_values?;
        let rustflags: Vec<(TemplateItemId, String)> = template
            .iter_items()
            .filter_map(|(id, value)| format_rustflag(id, value).map(|flag| (id, flag)))
            .collect();
        if rustflags.is_empty() && build_values.is_empty() {
            return Ok(self);
        }

        let build = self
            .document
            .entry("build")
            .or_insert(table())
            .as_table_like_mut()
            .ok_or_else(|| anyhow::anyhow!("The build item in config.toml is not a table"))?;

        for (key, build_value) in build_values {
            let build_item = build.entry(key).or_insert(value(build_value.clone()));
            if build_item.as_str() != Some(build_value.as_str()) {
                let value = build_item
                    .as_value_mut()
                    .ok_or_else(|| anyhow::anyhow!("build.{key} in config.toml is not a string"))?;
                let decor = value.decor().clone();
                *value = Value::String(Formatted::new(build_value));
                *value.decor_mut() = decor;
            }
        }

        if rustflags.is_empty() {
            return Ok(self);
        }
        let flags = build.entry("rustflags").or_insert(value(Array::new()));

        // build.rustflags can be either a string or an array of strings
        if let Some(array) = flags.as_array_mut() {
            edit_rustflags(array, &rustflags);
        } else if let Some(val) = flags.as_value_mut().filter(|v| v.is_str()) {
            let new_value = edit_rustflags_string(val.as_str().unwrap_or_default(), &rustflags);
            let decor = val.decor().clone();
            *val = Value::String(Formatted::new(new_value));
            *val.decor_mut() = decor;
        } else {
            return Err(anyhow::anyhow!(
                "build.rustflags in config.toml is not a string or an array"
            ));
        }

        Ok(self)
    }

    pub fn write(self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.path.parent().expect("Missing config.toml parent"))
            .context("Cannot create config.toml parent directory")?;
        std::fs::write(&self.path, self.document.to_string())
            .context("Cannot write config.toml manifest")?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum RustflagValueKind {
    String,
    Int,
}

#[derive(Clone, Copy)]
enum ConfigItemEncoding {
    BuildString {
        key: &'static str,
    },
    Rustflag {
        option: &'static str,
        key: &'static str,
        value_kind: RustflagValueKind,
    },
}

#[derive(Clone, Copy)]
struct ConfigItemDescriptor {
    id: TemplateItemId,
    encoding: ConfigItemEncoding,
}

const CONFIG_ITEMS: [ConfigItemDescriptor; 4] = [
    ConfigItemDescriptor {
        id: TemplateItemId::CompilerWrapper,
        encoding: ConfigItemEncoding::BuildString {
            key: "rustc-wrapper",
        },
    },
    ConfigItemDescriptor {
        id: TemplateItemId::TargetCpuInstructionSet,
        encoding: ConfigItemEncoding::Rustflag {
            option: "-C",
            key: "target-cpu=",
            value_kind: RustflagValueKind::String,
        },
    },
    ConfigItemDescriptor {
        id: TemplateItemId::FrontendThreads,
        encoding: ConfigItemEncoding::Rustflag {
            option: "-Z",
            key: "threads=",
            value_kind: RustflagValueKind::Int,
        },
    },
    ConfigItemDescriptor {
        id: TemplateItemId::Linker,
        encoding: ConfigItemEncoding::Rustflag {
            option: "-C",
            key: "link-arg=-fuse-ld=",
            value_kind: RustflagValueKind::String,
        },
    },
];

fn config_item_descriptor(id: TemplateItemId) -> Option<&'static ConfigItemDescriptor> {
    CONFIG_ITEMS.iter().find(|descriptor| descriptor.id == id)
}

fn format_rustflag(id: TemplateItemId, value: &TomlValue) -> Option<String> {
    let descriptor = config_item_descriptor(id)?;
    let ConfigItemEncoding::Rustflag { option, key, .. } = descriptor.encoding else {
        return None;
    };
    let value = toml_value_to_string(value);
    Some(format!("{option}{key}{value}"))
}

fn format_split_rustflag_argument(id: TemplateItemId, flag: &str) -> Option<String> {
    let descriptor = config_item_descriptor(id)?;
    let ConfigItemEncoding::Rustflag { option, key, .. } = descriptor.encoding else {
        return None;
    };
    let value = flag.strip_prefix(option)?.strip_prefix(key)?;
    Some(format!("{key}{value}"))
}

fn toml_value_to_string(value: &TomlValue) -> String {
    match value {
        TomlValue::String(value) => value.clone(),
        TomlValue::Int(value) => value.to_string(),
        TomlValue::Bool(value) => value.to_string(),
    }
}

fn parse_rustflag(flag: &str) -> Option<TemplateItemId> {
    CONFIG_ITEMS.iter().find_map(|descriptor| {
        let ConfigItemEncoding::Rustflag {
            option,
            key,
            value_kind,
        } = descriptor.encoding
        else {
            return None;
        };
        let value = flag.strip_prefix(option)?.strip_prefix(key)?;
        parse_rustflag_value(value, value_kind)?;
        Some(descriptor.id)
    })
}

fn parse_split_rustflag(option: &str, argument: &str) -> Option<TemplateItemId> {
    CONFIG_ITEMS.iter().find_map(|descriptor| {
        let ConfigItemEncoding::Rustflag {
            option: expected_option,
            key,
            value_kind,
        } = descriptor.encoding
        else {
            return None;
        };
        if option != expected_option {
            return None;
        }
        let value = argument.strip_prefix(key)?;
        parse_rustflag_value(value, value_kind)?;
        Some(descriptor.id)
    })
}

fn parse_rustflag_value(value: &str, value_kind: RustflagValueKind) -> Option<TomlValue> {
    match value_kind {
        RustflagValueKind::String => Some(TomlValue::String(value.to_string())),
        RustflagValueKind::Int => Some(TomlValue::Int(value.parse().ok()?)),
    }
}

#[derive(Debug)]
struct RustflagMatch {
    id: TemplateItemId,
    start: usize,
    token_count: usize,
}

fn parse_rustflags(flags: &[&str]) -> Vec<RustflagMatch> {
    let mut matches = Vec::new();
    let mut index = 0;
    while index < flags.len() {
        if let Some(id) = parse_rustflag(flags[index]) {
            matches.push(RustflagMatch {
                id,
                start: index,
                token_count: 1,
            });
            index += 1;
        } else if let Some(id) = flags
            .get(index + 1)
            .and_then(|argument| parse_split_rustflag(flags[index], argument))
        {
            matches.push(RustflagMatch {
                id,
                start: index,
                token_count: 2,
            });
            index += 2;
        } else {
            index += 1;
        }
    }
    matches
}

fn edit_rustflags(array: &mut Array, updates: &[(TemplateItemId, String)]) {
    let matches = {
        let existing: Vec<_> = array
            .iter()
            .map(|value| value.as_str().unwrap_or_default())
            .collect();
        parse_rustflags(&existing)
    };
    let mut replaced = HashSet::new();
    for flag in matches.into_iter().rev() {
        if let Some((_, new_value)) = updates.iter().find(|(id, _)| *id == flag.id) {
            let replacement = if flag.token_count == 1 {
                new_value.clone()
            } else {
                format_split_rustflag_argument(flag.id, new_value)
                    .expect("Parsed rustflag is missing from the descriptor registry")
            };
            let item = array
                .get_mut(flag.start + flag.token_count - 1)
                .expect("Parsed rustflag index is outside of the source array");
            let decor = item.decor().clone();
            let mut replacement = Value::String(Formatted::new(replacement));
            *replacement.decor_mut() = decor;
            *item = replacement;
            replaced.insert(flag.id);
        }
    }
    for (id, arg) in updates {
        if !replaced.contains(id) {
            array.push(Value::String(Formatted::new(arg.clone())));
        }
    }
}

/// Cargo splits a string-valued `build.rustflags` on whitespace, so it is edited as a flag array.
fn edit_rustflags_string(original: &str, updates: &[(TemplateItemId, String)]) -> String {
    let mut array: Array = original.split_ascii_whitespace().collect();
    edit_rustflags(&mut array, updates);
    array
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn config_path_from_manifest_path(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .map(|p| p.join(".cargo").join("config.toml"))
        .expect("Manifest path has no parent")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::str::FromStr;

    use toml_edit::DocumentMut;

    use crate::template::TemplateBuilder;
    use crate::workspace::manifest::BuiltinProfile;
    use crate::{CargoConfig, Template, TemplateItemId, TomlValue};

    use super::{
        CONFIG_ITEMS, ConfigItemEncoding, RustflagValueKind, config_item_descriptor,
        format_rustflag, parse_rustflag,
    };

    #[test]
    fn config_item_descriptors_are_unique_and_rustflags_round_trip() {
        let mut ids = HashSet::new();
        for descriptor in CONFIG_ITEMS {
            assert!(ids.insert(descriptor.id));
            assert_eq!(
                config_item_descriptor(descriptor.id).map(|item| item.id),
                Some(descriptor.id)
            );

            let ConfigItemEncoding::Rustflag { value_kind, .. } = descriptor.encoding else {
                continue;
            };
            let value = match value_kind {
                RustflagValueKind::String => TomlValue::String("sample".to_string()),
                RustflagValueKind::Int => TomlValue::Int(4),
            };
            let flag = format_rustflag(descriptor.id, &value).unwrap();
            assert_eq!(parse_rustflag(&flag), Some(descriptor.id));
        }
    }

    #[test]
    fn create_rustflags() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_empty_config().apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = ["-Ctarget-cpu=native"]
        "###);
    }

    #[test]
    fn append_to_array_rustflags() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = ["-Cbar=foo"]
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
    [build]
    rustflags = ["-Cbar=foo", "-Ctarget-cpu=native"]
    "###);
    }

    #[test]
    fn ignore_existing_entry() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "foo")]);
        let config = create_config(
            r#"
[build]
rustflags = ["-Ctarget-cpu=foo"]
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = ["-Ctarget-cpu=foo"]
        "###);
    }

    #[test]
    fn append_to_empty_string_rustflags() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = ""
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
            [build]
            rustflags = "-Ctarget-cpu=native"
            "###);
    }

    #[test]
    fn append_to_string_rustflags() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = "-Cfoo=bar"
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = "-Cfoo=bar -Ctarget-cpu=native"
        "###);
    }

    #[test]
    fn append_to_string_rustflags_keep_formatting() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = "-Cfoo=bar" # Foo
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = "-Cfoo=bar -Ctarget-cpu=native" # Foo
        "###);
    }

    #[test]
    fn replace_rustflag_value() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = [
    # Foo
    "-Ctarget-cpu=foo", # Foo
    "-Cbar=baz", # Foo
]
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"

        [build]
        rustflags = [
            # Foo
            "-Ctarget-cpu=native", # Foo
            "-Cbar=baz", # Foo
        ]
        "###);
    }

    #[test]
    fn replace_rustflag_value_in_string() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = "--cfg 'foo=bar baz'  -Ctarget-cpu=generic"
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = "--cfg 'foo=bar baz' -Ctarget-cpu=native"
        "###);
    }

    #[test]
    fn replace_linker_without_overwriting_other_link_args() {
        let template = create_template(&[(TemplateItemId::Linker, "mold")]);
        let config = create_config(
            r#"
[build]
rustflags = ["-Clink-arg=-Wl,--as-needed", "-Clink-arg=-fuse-ld=lld"]
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = ["-Clink-arg=-Wl,--as-needed", "-Clink-arg=-fuse-ld=mold"]
        "###);
    }

    #[test]
    fn replace_split_rustflag_in_array() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = ["-C", "target-cpu=generic", "--cfg", "x"]
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = ["-C", "target-cpu=native", "--cfg", "x"]
        "###);
    }

    #[test]
    fn replace_split_rustflag_in_string() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_config(
            r#"
[build]
rustflags = "-C target-cpu=generic -Cforce-frame-pointers=yes"
"#,
        );
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = "-C target-cpu=native -Cforce-frame-pointers=yes"
        "###);
    }

    #[test]
    fn create_compiler_wrapper() {
        let template = create_template(&[(TemplateItemId::CompilerWrapper, "sccache")]);
        let config = create_empty_config().apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustc-wrapper = "sccache"
        "###);
    }

    #[test]
    fn compiler_wrapper_coexists_with_rustflags() {
        let template = create_template(&[
            (TemplateItemId::CompilerWrapper, "sccache"),
            (TemplateItemId::TargetCpuInstructionSet, "native"),
        ]);
        let config = create_empty_config().apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustc-wrapper = "sccache"
        rustflags = ["-Ctarget-cpu=native"]
        "###);
    }

    #[test]
    fn replace_compiler_wrapper_keep_formatting() {
        let template = create_template(&[(TemplateItemId::CompilerWrapper, "sccache")]);
        let config = create_config(
            r#"
[build]
rustc-wrapper = "old-wrapper" # Wrapper
"#,
        );
        let config = config.apply_template(&template).unwrap();
        let config = config.apply_template(&template).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustc-wrapper = "sccache" # Wrapper
        "###);
    }

    fn create_template(items: &[(TemplateItemId, &str)]) -> Template {
        let mut builder = TemplateBuilder::new(BuiltinProfile::Release);
        for (id, value) in items {
            builder = builder.item(*id, TomlValue::String(value.to_string()));
        }
        builder.build()
    }

    fn create_config(text: &str) -> CargoConfig {
        CargoConfig {
            path: Default::default(),
            document: DocumentMut::from_str(text).unwrap(),
        }
    }

    fn create_empty_config() -> CargoConfig {
        CargoConfig {
            path: Default::default(),
            document: Default::default(),
        }
    }
}
