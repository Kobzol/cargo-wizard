use crate::utils::{init_cargo_project, CargoProject};

#[test]
fn dialog_fast_compile_to_dev() -> anyhow::Result<()> {
    let project = init_cargo_project()?;
    apply_profile(&project, 0, 0)?;

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

    apply_profile(&project, 0, 1)?;

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
        .release_template()
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

    apply_profile(&project, 0, 2)?;

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

    let mut terminal = project.cmd(&[]).start_terminal()?;
    terminal.key_enter()?;
    // Find "Custom profile option"
    terminal.key_down()?;
    terminal.key_down()?;
    terminal.key_enter()?;
    // Enter profile name
    terminal.line("custom1")?;
    // Customize template
    terminal.key_enter()?;
    // Confirm diff
    terminal.line("y")?;
    terminal.expect("Template FastCompile applied to profile custom1")?;
    terminal.wait()?;

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

struct DialogBuilder {
    profile_index: u32,
    template_index: u32,
    accept_diff: bool,
}

impl Default for DialogBuilder {
    fn default() -> Self {
        Self {
            profile_index: 0,
            template_index: 0,
            accept_diff: true,
        }
    }
}

impl DialogBuilder {
    fn profile(mut self, index: u32) -> Self {
        self.profile_index = index;
        self
    }

    fn template(mut self, index: u32) -> Self {
        self.template_index = index;
        self
    }

    fn release_template(self) -> Self {
        self.template(1)
    }

    fn accept_diff(mut self, value: bool) -> Self {
        self.accept_diff = value;
        self
    }

    fn run(self, project: &CargoProject) -> anyhow::Result<()> {
        let mut terminal = project.cmd(&[]).start_terminal()?;
        // Select template
        for _ in 0..self.template_index {
            terminal.key_down()?;
        }
        terminal.key_enter()?;
        // Select profile
        for _ in 0..self.profile_index {
            terminal.key_down()?;
        }
        terminal.key_enter()?;
        // Customize template
        terminal.key_enter()?;
        // Handle diff
        if self.accept_diff {
            terminal.line("y")?;
        } else {
            terminal.line("n")?;
        }
        terminal.wait()
    }
}

fn apply_fast_runtime_to_release(project: &CargoProject) -> anyhow::Result<()> {
    DialogBuilder::default()
        .release_template()
        .profile(1)
        .run(project)
}

fn apply_profile(
    project: &CargoProject,
    template_index: u32,
    profile_index: u32,
) -> anyhow::Result<()> {
    DialogBuilder::default()
        .template(template_index)
        .profile(profile_index)
        .run(project)
}
