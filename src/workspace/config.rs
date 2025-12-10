use std::collections::{HashMap, HashSet};
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
        let rustflags: Vec<String> = template
            .iter_items()
            .filter_map(|(id, value)| {
                let value = match value {
                    TomlValue::String(value) => value.clone(),
                    TomlValue::Int(value) => value.to_string(),
                    TomlValue::Bool(value) => value.to_string(),
                };
                match id {
                    TemplateItemId::TargetCpuInstructionSet => {
                        Some(format!("-Ctarget-cpu={value}"))
                    }
                    TemplateItemId::FrontendThreads => Some(format!("-Zthreads={value}")),
                    TemplateItemId::Linker => Some(format!("-Clink-arg=-fuse-ld={value}")),
                    TemplateItemId::DebugInfo
                    | TemplateItemId::Strip
                    | TemplateItemId::Lto
                    | TemplateItemId::CodegenUnits
                    | TemplateItemId::Panic
                    | TemplateItemId::OptimizationLevel
                    | TemplateItemId::CodegenBackend
                    | TemplateItemId::Incremental
                    | TemplateItemId::SplitDebugInfo => None,
                }
            })
            .collect();
        if rustflags.is_empty() {
            return Ok(self);
        }

        let build = self
            .document
            .entry("build")
            .or_insert(table())
            .as_table_mut()
            .ok_or_else(|| anyhow::anyhow!("The build item in config.toml is not a table"))?;
        let flags = build.entry("rustflags").or_insert(value(Array::new()));

        let flag_map: HashMap<_, _> = rustflags
            .iter()
            .filter_map(|rustflag| {
                let (key, value) = rustflag.split_once('=')?;
                Some((key.to_string(), value.to_string()))
            })
            .collect();

        // build.rustflags can be either a string or an array of strings
        if let Some(array) = flags.as_array_mut() {
            // Find flags with the same key (e.g. -Ckey=val) and replace their values, to avoid
            // duplicating the keys.
            for item in array.iter_mut() {
                if let Some(val) = item.as_str()
                    && let Some((key, _)) = val.split_once('=')
                    && let Some(new_value) = flag_map.get(key)
                {
                    let decor = item.decor().clone();
                    let mut new_value = Value::String(Formatted::new(format!("{key}={new_value}")));
                    *new_value.decor_mut() = decor;
                    *item = new_value;
                }
            }

            let existing_flags: HashSet<String> = array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            for arg in rustflags {
                if !existing_flags.contains(&arg) {
                    array.push(Value::String(Formatted::new(arg)));
                }
            }
        } else if let Some(val) = flags.as_value_mut().filter(|v| v.is_str()) {
            let flattened_flags = rustflags.join(" ");
            let mut original_value = val.as_str().unwrap_or_default().to_string();
            if !original_value.ends_with(' ') && !original_value.is_empty() {
                original_value.push(' ');
            }
            original_value.push_str(&flattened_flags);
            let decor = val.decor().clone();
            *val = Value::String(Formatted::new(original_value));
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
