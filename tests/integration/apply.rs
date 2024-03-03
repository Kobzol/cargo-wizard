use crate::utils::{init_cargo_project, Cmd, OutputExt};

#[test]
fn apply_explicit_manifest_path() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    let manifest_path = "crates/inner/Cargo.toml";
    project.file(
        manifest_path,
        r#"
[package]
name = "foo"
version = "0.1.0"
edition = "2021"
"#,
    );
    project
        .cmd(&[
            "apply",
            "dev",
            "fast-compile",
            "--manifest-path",
            manifest_path,
        ])
        .run()?
        .assert_ok();
    insta::assert_snapshot!(project.read(manifest_path), @r###"

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
fn resolve_workspace_root() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.file(
        "bar/Cargo.toml",
        r#"
[package]
name = "bar"
version = "0.1.0"
edition = "2021"
"#,
    );
    project.file("bar/src/lib.rs", "");
    project.manifest(
        r#"
[workspace]
members = ["bar"]
"#,
    );

    project
        .cmd(&["apply", "dev", "fast-compile"])
        .cwd(&project.path("bar"))
        .run()?
        .assert_ok();
    insta::assert_snapshot!(project.read_manifest(), @r###"
    [workspace]
    members = ["bar"]

    [profile.dev]
    debug = 0
"###);

    Ok(())
}

#[test]
fn apply_missing_builtin() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    project
        .cmd(&["apply", "dev", "fast-compile"])
        .run()?
        .assert_ok();
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
