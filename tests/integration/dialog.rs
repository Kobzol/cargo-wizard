use crate::utils::terminal::Terminal;
use crate::utils::{init_cargo_project, CargoProject};

#[test]
fn dialog_fast_compile_to_dev() -> anyhow::Result<()> {
    let project = init_cargo_project()?;
    apply_profile(&project, "FastCompile", "dev")?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]
    debug = 0
    "###);

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Clink-arg=-fuse-ld=lld"]
    "###);

    Ok(())
}

#[test]
fn dialog_min_size_to_release() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply_profile(&project, "MinSize", "release")?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.release]
    debug = 0
    strip = true
    lto = true
    opt-level = "z"
    codegen-units = 1
    panic = "abort"
    "###);

    assert!(!project.file_exists(project.config_path()));

    Ok(())
}

#[test]
fn dialog_deny_diff() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    DialogBuilder::default()
        .profile_release()
        .accept_diff(false)
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"
    "###);

    assert!(!project.file_exists(project.config_path()));

    Ok(())
}

#[test]
fn dialog_find_custom_profile() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.manifest(
        r#"
[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.custom1]
inherits = "dev"
debug = 1
"#,
    );

    let mut terminal = project.cmd(&[]).start_terminal()?;
    terminal.select_line("custom1")?;

    Ok(())
}

#[test]
fn dialog_fast_compile_to_custom_profile() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.manifest(
        r#"
[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.custom1]
inherits = "dev"
debug = 1
"#,
    );

    DialogBuilder::default()
        .template("FastCompile")
        .profile("custom1")
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom1]
    inherits = "dev"
    debug = 0
    "###);

    Ok(())
}

#[test]
fn dialog_fast_compile_to_new_profile() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    DialogBuilder::default()
        .template("FastCompile")
        .create_profile("custom1")
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom1]
    inherits = "dev"
    debug = 0
    "###);

    Ok(())
}

#[test]
fn dialog_empty_profile_name() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    let mut terminal = DialogBuilder::default().start(&project)?;
    terminal.select_line("<Create a new profile>")?;
    terminal.line("")?;
    terminal.expect("Profile name must not be empty")
}

#[test]
fn dialog_invalid_profile_name() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    let mut terminal = DialogBuilder::default().start(&project)?;
    terminal.select_line("<Create a new profile>")?;
    terminal.line("#")?;
    terminal.expect("Profile name may contain only letters, numbers, underscore and hyphen")
}

#[test]
fn dialog_create_config() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply_fast_runtime_to_release(&project)?;

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Ctarget-cpu=native"]
    "###);

    Ok(())
}

#[test]
fn dialog_append_to_config() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.config(
        r#"
[build]
rustflags = ["-Ccodegen-units=1"]
"#,
    );

    apply_fast_runtime_to_release(&project)?;

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Ccodegen-units=1", "-Ctarget-cpu=native"]
    "###);

    Ok(())
}

#[test]
fn dialog_skip_existing_flags_in_config() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.config(
        r#"
[build]
rustflags = ["-Ctarget-cpu=native"]
"#,
    );

    apply_fast_runtime_to_release(&project)?;

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Ctarget-cpu=native"]
    "###);

    Ok(())
}

#[test]
fn dialog_codegen_backend_add_cargo_features() -> anyhow::Result<()> {
    let project = init_cargo_project()?.disable_check_on_drop();

    DialogBuilder::default()
        .customize_item("Codegen backend", "Cranelift")
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    cargo-features = ["codegen-backend"]
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]
    debug = 0
    codegen-backend = "cranelift"
    "###);

    Ok(())
}

#[test]
fn dialog_codegen_backend_append_to_cargo_features() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?.disable_check_on_drop();
    project.manifest(
        r#"cargo-features = []

[package]
name = "foo"
version = "0.1.0"
edition = "2021"
"#,
    );

    DialogBuilder::default()
        .customize_item("Codegen backend", "Cranelift")
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    cargo-features = ["codegen-backend"]

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]
    debug = 0
    codegen-backend = "cranelift"
    "###);

    Ok(())
}

#[test]
fn dialog_check_nightly_output() -> anyhow::Result<()> {
    let project = init_cargo_project()?.disable_check_on_drop();

    DialogBuilder::default()
        .profile_release()
        .customize_item("Codegen backend", "Cranelift")
        .with_final_check("cargo +nightly <cmd>")
        .with_final_check("You will have to use a nightly compiler")
        .run(&project)?;

    Ok(())
}

#[test]
fn dialog_codegen_backend_nightly_mark() -> anyhow::Result<()> {
    let project = init_cargo_project()?.disable_check_on_drop();

    DialogBuilder::default()
        .profile_release()
        .customize_item("Codegen backend *", "Cranelift")
        .run(&project)?;

    Ok(())
}

#[test]
fn dialog_fast_compile_nightly() -> anyhow::Result<()> {
    let project = init_cargo_project()?.disable_check_on_drop();

    DialogBuilder::default()
        .nightly()
        .customize_item(
            "Amount of frontend threads",
            CustomValue::Custom("4".to_string()),
        )
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    cargo-features = ["codegen-backend"]
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]
    debug = 0
    codegen-backend = "cranelift"
    "###);

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Clink-arg=-fuse-ld=lld", "-Zthreads=4"]
    "###);

    Ok(())
}

#[test]
fn dialog_unset_item() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    DialogBuilder::default()
        .template("FastRuntime")
        .customize_item("Panic", "<Unset value>")
        .run(&project)?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]
    lto = true
    codegen-units = 1
    "###);

    Ok(())
}

enum CustomValue {
    Constant(String),
    Custom(String),
}

impl<'a> From<&'a str> for CustomValue {
    fn from(value: &'a str) -> Self {
        Self::Constant(value.to_string())
    }
}

struct DialogBuilder {
    profile: String,
    created_profile: Option<String>,
    template: String,
    nightly: bool,
    accept_diff: bool,
    customized_items: Vec<(String, CustomValue)>,
    final_checks: Vec<String>,
}

impl Default for DialogBuilder {
    fn default() -> Self {
        Self {
            profile: "dev".to_string(),
            created_profile: None,
            template: "FastCompile".to_string(),
            nightly: false,
            accept_diff: true,
            customized_items: vec![],
            final_checks: vec![],
        }
    }
}

impl DialogBuilder {
    fn template(mut self, name: &str) -> Self {
        self.template = name.to_string();
        self
    }

    fn profile(mut self, name: &str) -> Self {
        self.profile = name.to_string();
        self
    }

    fn create_profile(mut self, name: &str) -> Self {
        self.created_profile = Some(name.to_string());
        self.profile = "<Create a new profile>".to_string();
        self
    }

    fn profile_release(self) -> Self {
        self.profile("release")
    }

    fn nightly(mut self) -> Self {
        self.nightly = true;
        self
    }

    fn accept_diff(mut self, value: bool) -> Self {
        self.accept_diff = value;
        self
    }

    fn customize_item<V: Into<CustomValue>>(mut self, name: &str, value: V) -> Self {
        self.customized_items.push((name.to_string(), value.into()));
        self
    }

    fn with_final_check(mut self, name: &str) -> Self {
        self.final_checks.push(name.to_string());
        self
    }

    fn start(&self, project: &CargoProject) -> anyhow::Result<Terminal> {
        let nightly = match self.nightly {
            true => "on",
            false => "off",
        };
        let terminal = project
            .cmd(&[&format!("--nightly={nightly}")])
            .start_terminal()?;
        Ok(terminal)
    }

    fn run(self, project: &CargoProject) -> anyhow::Result<()> {
        let mut terminal = self.start(project)?;
        // Select profile
        terminal.expect("Select the profile that you want to update/create")?;
        terminal.select_line(&self.profile)?;
        if let Some(ref custom_profile) = self.created_profile {
            terminal.expect("Select profile name")?;
            terminal.line(custom_profile)?;
        }
        // Select template
        terminal.expect("Select the template that you want to apply")?;
        terminal.select_line(&self.template)?;
        terminal.expect("Select items to modify or confirm the template")?;
        // Customize template
        for (name, value) in self.customized_items {
            terminal.select_line(&name)?;
            match value {
                // Select from list
                CustomValue::Constant(value) => {
                    terminal.select_line(&value)?;
                }
                // Enter custom value
                CustomValue::Custom(value) => {
                    terminal.select_line("Custom value")?;
                    terminal.line(&value)?;
                }
            }
        }
        // Confirm template
        terminal.key_enter()?;
        terminal.expect("Do you want to apply the above diff")?;

        let profile_name = self.created_profile.unwrap_or(self.profile);

        // Handle diff
        if self.accept_diff {
            terminal.line("y")?;
            terminal.expect(&format!(
                "Template {} applied to profile {profile_name}",
                self.template
            ))?;
            for check in self.final_checks {
                terminal.expect(&check)?;
            }
            terminal.wait()?;
        } else {
            terminal.line("n")?;
            terminal.expect("Select items to modify or confirm the template")?;
        }
        Ok(())
    }
}

fn apply_fast_runtime_to_release(project: &CargoProject) -> anyhow::Result<()> {
    DialogBuilder::default()
        .template("FastRuntime")
        .profile_release()
        .run(project)
}

fn apply_profile(project: &CargoProject, template: &str, profile: &str) -> anyhow::Result<()> {
    DialogBuilder::default()
        .template(template)
        .profile(profile)
        .run(project)
}
