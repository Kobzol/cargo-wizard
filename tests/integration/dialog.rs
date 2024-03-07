use crate::utils::{init_cargo_project, CargoProject};

// TODO: decline diff

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

fn apply_fast_runtime_to_release(project: &CargoProject) -> anyhow::Result<()> {
    apply_profile(project, 1, 1)
}

fn apply_profile(
    project: &CargoProject,
    template_index: u64,
    profile_index: u64,
) -> anyhow::Result<()> {
    let mut terminal = project.cmd(&[]).start_terminal()?;
    // Select template
    for _ in 0..template_index {
        terminal.key_down()?;
    }
    terminal.key_enter()?;
    // Select profile
    for _ in 0..profile_index {
        terminal.key_down()?;
    }
    terminal.key_enter()?;
    // Customize template
    terminal.key_enter()?;
    // Confirm diff
    terminal.line("y")?;
    terminal.wait()?;

    Ok(())
}
