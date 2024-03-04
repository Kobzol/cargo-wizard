use crate::utils::init_cargo_project;

#[test]
fn dialog_fast_compile_dev() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    let mut terminal = project.cmd(&[]).start_terminal()?;
    terminal.expect("Select the template that you want to apply")?;
    terminal.key_enter()?;
    terminal.expect("Select the profile that you want to update/create")?;
    terminal.key_enter()?;
    terminal.line("y")?;
    terminal.expect("applied to profile")?;
    terminal.wait()?;

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
fn dialog_fast_compile_release() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    let mut terminal = project.cmd(&[]).start_terminal()?;
    terminal.expect("Select the template that you want to apply")?;
    terminal.key_enter()?;
    terminal.expect("Select the profile that you want to update/create")?;
    terminal.key_down()?;
    terminal.key_enter()?;
    terminal.line("y")?;
    terminal.expect("applied to profile")?;
    terminal.wait()?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.release]
    debug = 0
    "###);

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

[profile.custom]
inherits = "dev"
debug = 1
"#,
    );

    let mut terminal = project.cmd(&[]).start_terminal()?;
    terminal.expect("Select the template that you want to apply")?;
    terminal.key_enter()?;
    terminal.expect("Select the profile that you want to update/create")?;
    terminal.expect("custom")?;

    Ok(())
}

#[test]
fn dialog_fast_compile_custom_profile() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    let mut terminal = project.cmd(&[]).start_terminal()?;
    terminal.expect("Select the template that you want to apply")?;
    terminal.key_enter()?;
    terminal.expect("Select the profile that you want to update/create")?;
    // Find "Custom profile option"
    terminal.key_down()?;
    terminal.key_down()?;
    terminal.key_enter()?;
    // Enter profile name
    terminal.line("custom")?;
    terminal.line("y")?;
    terminal.expect("applied to profile")?;
    terminal.wait()?;

    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom]
    inherits = "dev"
    debug = 0
    "###);

    Ok(())
}
