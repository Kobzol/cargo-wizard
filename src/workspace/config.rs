use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{Template, TemplateItemId, TomlValue};
use anyhow::Context;
use toml_edit::{Array, DocumentMut, Item, Value, table, value};

/// Template items stored in `build.rustflags` as `{option}{key}{value}`, or as the two
/// arguments `{option} {key}{value}`.
const RUSTFLAGS: [(TemplateItemId, &str, &str); 3] = [
    (TemplateItemId::TargetCpuInstructionSet, "-C", "target-cpu="),
    (TemplateItemId::FrontendThreads, "-Z", "threads="),
    (TemplateItemId::Linker, "-C", "link-arg=-fuse-ld="),
];

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

    /// Adds the supported workspace-global Cargo config values to a template.
    pub fn load_into_template(&self, template: &mut Template) -> anyhow::Result<()> {
        let Some(build) = self.document.get("build").and_then(Item::as_table_like) else {
            return Ok(());
        };
        if let Some(wrapper) = build.get("rustc-wrapper") {
            let wrapper = wrapper.as_str().ok_or_else(|| {
                anyhow::anyhow!("build.rustc-wrapper in config.toml is not a string")
            })?;
            template.insert_item(TemplateItemId::CompilerWrapper, TomlValue::string(wrapper));
        }
        let tokens: Vec<&str> = match build.get("rustflags") {
            Some(Item::Value(Value::Array(array))) => array
                .iter()
                .map(|flag| {
                    flag.as_str().ok_or_else(|| {
                        anyhow::anyhow!("build.rustflags in config.toml must contain only strings")
                    })
                })
                .collect::<anyhow::Result<_>>()?,
            Some(Item::Value(Value::String(flags))) => {
                flags.value().split_ascii_whitespace().collect()
            }
            Some(_) => anyhow::bail!("build.rustflags in config.toml is not a string or an array"),
            None => Vec::new(),
        };
        for flag in find_rustflags(&tokens) {
            template.insert_item(flag.id, flag.value);
        }
        Ok(())
    }

    /// Sets the template's config items. With `replace`, also removes the supported items that the
    /// template does not set.
    pub fn apply_template(mut self, template: &Template, replace: bool) -> anyhow::Result<Self> {
        let wrapper = template
            .get_item(TemplateItemId::CompilerWrapper)
            .map(|value| match value {
                TomlValue::String(value) => Ok(value.as_str()),
                _ => Err(anyhow::anyhow!(
                    "Compiler wrapper in the template must be a string"
                )),
            })
            .transpose()?;
        let rustflags: Vec<(TemplateItemId, String)> = template
            .iter_items()
            .filter_map(|(id, value)| {
                let (_, option, key) = RUSTFLAGS.iter().find(|(flag_id, ..)| *flag_id == id)?;
                let value = match value {
                    TomlValue::String(value) => value.clone(),
                    TomlValue::Int(value) => value.to_string(),
                    TomlValue::Bool(value) => value.to_string(),
                };
                Some((id, format!("{option}{key}{value}")))
            })
            .collect();
        if wrapper.is_none()
            && rustflags.is_empty()
            && (!replace || !self.document.contains_key("build"))
        {
            return Ok(self);
        }

        let build = self
            .document
            .entry("build")
            .or_insert(table())
            .as_table_like_mut()
            .ok_or_else(|| anyhow::anyhow!("The build item in config.toml is not a table"))?;

        match wrapper {
            Some(wrapper) => {
                let item = build.entry("rustc-wrapper").or_insert(value(wrapper));
                let item = item.as_value_mut().ok_or_else(|| {
                    anyhow::anyhow!("build.rustc-wrapper in config.toml is not a string")
                })?;
                set_string(item, wrapper);
            }
            None if replace => {
                build.remove("rustc-wrapper");
            }
            None => {}
        }

        if rustflags.is_empty() && (!replace || !build.contains_key("rustflags")) {
            return Ok(self);
        }
        // build.rustflags can be either a string or an array of strings
        match build.entry("rustflags").or_insert(value(Array::new())) {
            Item::Value(Value::Array(array)) => edit_rustflags(array, &rustflags, replace),
            Item::Value(value @ Value::String(_)) => {
                // Cargo splits a string-valued build.rustflags on whitespace.
                let original = value.as_str().unwrap_or_default();
                let mut array: Array = original.split_ascii_whitespace().collect();
                edit_rustflags(&mut array, &rustflags, replace);
                let tokens: Vec<_> = array.iter().filter_map(Value::as_str).collect();
                if !tokens.iter().copied().eq(original.split_ascii_whitespace()) {
                    set_string(value, &tokens.join(" "));
                }
            }
            _ => anyhow::bail!("build.rustflags in config.toml is not a string or an array"),
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

/// Replaces a string value, keeping its comments and whitespace.
fn set_string(item: &mut Value, new_value: &str) {
    if item.as_str() != Some(new_value) {
        let decor = item.decor().clone();
        *item = new_value.into();
        *item.decor_mut() = decor;
    }
}

/// A supported rustflag found at `tokens[start]`, split over two tokens if `split`.
struct FoundRustflag {
    id: TemplateItemId,
    value: TomlValue,
    start: usize,
    split: bool,
}

fn find_rustflags(tokens: &[&str]) -> Vec<FoundRustflag> {
    let mut found = Vec::new();
    let mut start = 0;
    while start < tokens.len() {
        let next = tokens.get(start + 1).copied().unwrap_or_default();
        let flag = RUSTFLAGS.iter().find_map(|&(id, option, key)| {
            let rest = tokens[start].strip_prefix(option)?;
            let split = rest.is_empty();
            let value = if split { next } else { rest }.strip_prefix(key)?;
            let value = match id {
                TemplateItemId::FrontendThreads => TomlValue::Int(value.parse().ok()?),
                _ => TomlValue::string(value),
            };
            Some(FoundRustflag {
                id,
                value,
                start,
                split,
            })
        });
        start += 1 + flag.as_ref().is_some_and(|flag| flag.split) as usize;
        found.extend(flag);
    }
    found
}

/// Replaces the existing flags of `updates` in place and appends the missing ones. With `replace`,
/// removes the supported flags that are not in `updates`.
fn edit_rustflags(array: &mut Array, updates: &[(TemplateItemId, String)], replace: bool) {
    let tokens: Vec<_> = array
        .iter()
        .map(|value| value.as_str().unwrap_or_default())
        .collect();
    let found = find_rustflags(&tokens);
    let mut updated = HashSet::new();
    for flag in found.into_iter().rev() {
        let end = flag.start + flag.split as usize;
        if let Some((_, new_flag)) = updates.iter().find(|(id, _)| *id == flag.id) {
            let option_len = if flag.split {
                array
                    .get(flag.start)
                    .and_then(Value::as_str)
                    .map_or(0, str::len)
            } else {
                0
            };
            let item = array
                .get_mut(end)
                .expect("Found rustflag is outside of the array");
            set_string(item, &new_flag[option_len..]);
            updated.insert(flag.id);
        } else if replace {
            for index in (flag.start..=end).rev() {
                array.remove(index);
            }
        }
    }
    for (id, flag) in updates {
        if !updated.contains(id) {
            array.push(flag.as_str());
        }
    }
}

pub fn config_path_from_manifest_path(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .map(|p| p.join(".cargo").join("config.toml"))
        .expect("Manifest path has no parent")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use toml_edit::DocumentMut;

    use crate::template::TemplateBuilder;
    use crate::workspace::manifest::BuiltinProfile;
    use crate::{CargoConfig, Template, TemplateItemId, TomlValue};

    #[test]
    fn create_rustflags() {
        let template = create_template(&[(TemplateItemId::TargetCpuInstructionSet, "native")]);
        let config = create_empty_config()
            .apply_template(&template, false)
            .unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
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
    fn replace_linker_without_overwriting_other_link_args() {
        let template = create_template(&[(TemplateItemId::Linker, "mold")]);
        let config = create_config(
            r#"
[build]
rustflags = ["-Clink-arg=-Wl,--as-needed", "-Clink-arg=-fuse-ld=lld"]
"#,
        );
        let config = config.apply_template(&template, false).unwrap();
        insta::assert_snapshot!(config.get_text(), @r###"
        [build]
        rustflags = ["-Clink-arg=-Wl,--as-needed", "-Clink-arg=-fuse-ld=mold"]
        "###);
    }

    #[test]
    fn create_compiler_wrapper() {
        let template = create_template(&[(TemplateItemId::CompilerWrapper, "sccache")]);
        let config = create_empty_config()
            .apply_template(&template, false)
            .unwrap();
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
        let config = create_empty_config()
            .apply_template(&template, false)
            .unwrap();
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
        let config = config.apply_template(&template, false).unwrap();
        let config = config.apply_template(&template, false).unwrap();
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
