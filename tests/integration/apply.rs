use crate::utils::{init_cargo_project, OutputExt};

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
        .run(&[
            "apply",
            "dev",
            "fast-compile",
            "--manifest-path",
            manifest_path,
        ])?
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
