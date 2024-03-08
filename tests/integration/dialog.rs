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

    Ok(())
}

#[test]
fn dialog_fast_compile_to_release() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply_profile(&project, "FastCompile", "release")?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.release]
    debug = 0
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
    terminal.key_enter()?;
    terminal.expect("custom1")?;

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

struct DialogBuilder {
    profile: String,
    created_profile: Option<String>,
    template: String,
    accept_diff: bool,
    customized_items: Vec<(String, String)>,
}

impl Default for DialogBuilder {
    fn default() -> Self {
        Self {
            profile: "dev".to_string(),
            created_profile: None,
            template: "FastCompile".to_string(),
            accept_diff: true,
            customized_items: vec![],
        }
    }
}

impl DialogBuilder {
    fn profile(mut self, name: &str) -> Self {
        self.profile = name.to_string();
        self
    }

    fn template(mut self, name: &str) -> Self {
        self.template = name.to_string();
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

    fn accept_diff(mut self, value: bool) -> Self {
        self.accept_diff = value;
        self
    }

    fn customize_item(mut self, name: &str, value: &str) -> Self {
        self.customized_items
            .push((name.to_string(), value.to_string()));
        self
    }

    fn run(self, project: &CargoProject) -> anyhow::Result<()> {
        let mut terminal = project.cmd(&[]).start_terminal()?;
        // Select template
        terminal.expect("Select the template that you want to apply")?;
        terminal.select_line(&self.template)?;
        // Select profile
        terminal.expect("Select the profile that you want to update/create")?;
        terminal.select_line(&self.profile)?;
        if let Some(ref custom_profile) = self.created_profile {
            terminal.expect("Select profile name")?;
            terminal.line(&custom_profile)?;
        }
        terminal.expect("Select items to modify or confirm the template")?;
        // Customize template
        for (name, value) in self.customized_items {
            terminal.select_line(&name)?;
            terminal.select_line(&value)?;
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
        } else {
            terminal.line("n")?;
        }
        terminal.wait()
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
