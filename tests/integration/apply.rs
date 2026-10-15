use crate::utils::{CargoProject, OutputExt, init_cargo_project};
use cargo_wizard::{
    ModificationResult, ModifiedWorkspace, Profile, Template, TemplateItemId, TomlValue,
    parse_workspace,
};

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
            "fast-compile",
            "dev",
            "--nightly=off",
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
        .cmd(&["apply", "fast-compile", "dev", "--nightly=off"])
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
    let project = init_cargo_project()?;

    apply(&project, "dev", "fast-compile")?;
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
fn apply_existing_builtin() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    project.manifest(
        r#"
[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.dev]
debug = 1
"#,
    );

    apply(&project, "dev", "fast-compile")?;
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
fn apply_missing_custom() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply(&project, "custom1", "fast-compile")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom1]
    inherits = "dev"
    opt-level = 0
    debug = 0
    strip = "none"
    lto = false
    codegen-units = 256
    incremental = true
    "###);

    Ok(())
}

#[test]
fn apply_existing_custom() -> anyhow::Result<()> {
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

    apply(&project, "custom1", "fast-compile")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom1]
    inherits = "dev"
    debug = 0
    opt-level = 0
    strip = "none"
    lto = false
    codegen-units = 256
    incremental = true
    "###);

    Ok(())
}

#[test]
fn apply_existing_keep_formatting() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;

    project.manifest(
        r#"
[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.dev]

lto =      "thin"

debug = 1   # Foo

codegen-units    = 10
"#,
    );

    apply(&project, "dev", "fast-compile")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"

    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]

    lto =      false

    debug = 0   # Foo

    codegen-units    = 256
    "###);

    Ok(())
}

#[test]
fn apply_fast_runtime_template() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply(&project, "custom", "fast-runtime")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom]
    inherits = "release"
    opt-level = 3
    debug = false
    strip = "none"
    lto = true
    codegen-units = 1
    incremental = false
    panic = "abort"
    "###);

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Ctarget-cpu=native"]
    "###);

    Ok(())
}

#[test]
fn apply_fast_runtime_template_to_dev_writes_release_defaults() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply(&project, "dev", "fast-runtime")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.dev]
    opt-level = 3
    debug = false
    lto = true
    codegen-units = 1
    incremental = false
    panic = "abort"
    "###);

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Ctarget-cpu=native"]
    "###);

    Ok(())
}

#[test]
fn apply_fast_runtime_template_to_release_skips_release_defaults() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply(&project, "release", "fast-runtime")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.release]
    lto = true
    codegen-units = 1
    panic = "abort"
    "###);

    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = ["-Ctarget-cpu=native"]
    "###);

    Ok(())
}

#[test]
fn apply_min_size_template() -> anyhow::Result<()> {
    let project = init_cargo_project()?;

    apply(&project, "custom", "min-size")?;
    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.custom]
    inherits = "release"
    opt-level = "z"
    debug = false
    strip = true
    lto = true
    codegen-units = 1
    incremental = false
    panic = "abort"
    "###);

    Ok(())
}

#[test]
fn load_current_settings_round_trips_without_changes() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?.disable_check_on_drop();
    project.manifest(
        r#"cargo-features = ["codegen-backend"]

[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.custom]
inherits = 'release' # base profile
opt-level = "s"
debug = 1
split-debuginfo = "packed"
strip = "debuginfo"
lto = "thin"
codegen-units = 4
panic = "abort"
incremental = false
codegen-backend = "cranelift"
"#,
    );
    project.config(
        r#"[build]
rustc-wrapper = "sccache"
rustflags = "-Ctarget-cpu=native  -Zthreads=4 -C link-arg=-fuse-ld=mold"
"#,
    );

    // Every item is loaded with its value, otherwise replacing would remove or rewrite it.
    let modified = edit_current(&project, &Profile::Custom("custom".into()), |template| {
        assert_eq!(template.iter_items().count(), 13);
        assert_eq!(
            template.get_item(TemplateItemId::FrontendThreads),
            Some(&TomlValue::Int(4))
        );
    })?;
    assert!(matches!(modified.manifest(), ModificationResult::NoChange));
    assert!(matches!(modified.config(), ModificationResult::NoChange));
    Ok(())
}

#[test]
fn config_only_current_settings_do_not_create_a_profile() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.config(
        r#"[build]
rustflags = ["-Ctarget-cpu=generic"]
"#,
    );

    let workspace = parse_workspace(&project.manifest_path())?;
    let template = workspace.load_template(&Profile::dev())?;
    for replace in [false, true] {
        let modified = workspace
            .clone()
            .apply(&Profile::dev(), &template, replace)?;
        assert!(matches!(modified.manifest(), ModificationResult::NoChange));
        assert!(matches!(modified.config(), ModificationResult::NoChange));
    }
    Ok(())
}

#[test]
fn edit_current_settings_removes_values_and_preserves_unknown_flags() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?.disable_check_on_drop();
    project.manifest(
        r#"[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.base]
inherits = "release"

[profile.custom]
inherits = "base"
debug = 1
lto = true
"#,
    );
    project.config(
        r#"[build]
rustc-wrapper = "sccache"
rustflags = "-Copt-level=2 -Ctarget-cpu=generic"
"#,
    );

    edit_current(&project, &Profile::Custom("custom".into()), |template| {
        template.remove_item(TemplateItemId::DebugInfo);
        template.insert_item(TemplateItemId::Lto, TomlValue::Bool(false));
        template.remove_item(TemplateItemId::CompilerWrapper);
        template.remove_item(TemplateItemId::TargetCpuInstructionSet);
    })?
    .write()?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.base]
    inherits = "release"

    [profile.custom]
    inherits = "base"
    lto = false
    "###);
    insta::assert_snapshot!(project.read_config(), @r###"
    [build]
    rustflags = "-Copt-level=2"
    "###);
    Ok(())
}

#[test]
fn edit_current_settings_handles_split_rustflags() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?.disable_check_on_drop();
    let native = || TomlValue::String("native".into());
    project.config(
        r#"[build]
rustflags = ["-C", "target-cpu=generic", "-C", "link-arg=-fuse-ld=lld", "-Copt-level=2"]
"#,
    );
    edit_current(&project, &Profile::dev(), |template| {
        template.insert_item(TemplateItemId::TargetCpuInstructionSet, native());
        template.remove_item(TemplateItemId::Linker);
    })?
    .write()?;
    insta::assert_snapshot!(project.read_config(), @r###"[build]
rustflags = ["-C", "target-cpu=native", "-Copt-level=2"]
"###);

    project.config(
        r#"[build]
rustflags = "-Z threads=4  -C target-cpu=generic -Copt-level=2"
"#,
    );
    edit_current(&project, &Profile::dev(), |template| {
        template.remove_item(TemplateItemId::FrontendThreads);
        template.insert_item(TemplateItemId::TargetCpuInstructionSet, native());
    })?
    .write()?;
    insta::assert_snapshot!(project.read_config(), @r###"[build]
rustflags = "-C target-cpu=native -Copt-level=2"
"###);
    Ok(())
}

#[test]
fn unsetting_last_codegen_backend_removes_feature_gate() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?.disable_check_on_drop();
    project.manifest(
        r#"cargo-features = ["codegen-backend"]

[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.dev]
codegen-backend = "cranelift"

[profile.release]
codegen-backend = "cranelift"
"#,
    );

    let unset_backend =
        |template: &mut Template| template.remove_item(TemplateItemId::CodegenBackend);
    edit_current(&project, &Profile::dev(), unset_backend)?.write()?;
    // The release profile still uses the feature.
    assert!(
        project
            .read_manifest()
            .starts_with(r#"cargo-features = ["codegen-backend"]"#)
    );
    edit_current(&project, &Profile::release(), unset_backend)?.write()?;
    insta::assert_snapshot!(project.read_manifest(), @r###"cargo-features = []

[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.dev]

[profile.release]
"###);
    Ok(())
}

#[test]
fn edit_current_settings_preserves_inline_tables() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?.disable_check_on_drop();
    project.manifest(
        r#"profile = { dev = { debug = 1, lto = true } }

[package]
name = "foo"
version = "0.1.0"
edition = "2021"
"#,
    );
    project.config(
        r#"build = { rustc-wrapper = "sccache", rustflags = ["-C", "target-cpu=generic"] }
"#,
    );

    edit_current(&project, &Profile::dev(), |template| {
        template.insert_item(TemplateItemId::DebugInfo, TomlValue::Int(2));
        template.insert_item(
            TemplateItemId::TargetCpuInstructionSet,
            TomlValue::String("native".into()),
        );
    })?
    .write()?;

    insta::assert_snapshot!(project.read_manifest(), @r###"profile = { dev = { debug = 2, lto = true } }

[package]
name = "foo"
version = "0.1.0"
edition = "2021"
"###);
    insta::assert_snapshot!(project.read_config(), @r###"build = { rustc-wrapper = "sccache", rustflags = ["-C", "target-cpu=native"] }
"###);
    Ok(())
}

#[test]
fn reused_current_template_preserves_custom_inheritance() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.manifest(
        r#"[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[profile.base]
inherits = "release"

[profile.source]
inherits = "base"
debug = 1
"#,
    );

    let workspace = parse_workspace(&project.manifest_path())?;
    let template = workspace.load_template(&Profile::Custom("source".to_string()))?;
    workspace
        .apply_template(&Profile::Custom("copy".to_string()), &template)?
        .write()?;

    insta::assert_snapshot!(project.read_manifest(), @r###"
    [package]
    name = "foo"
    version = "0.1.0"
    edition = "2021"

    [profile.base]
    inherits = "release"

    [profile.source]
    inherits = "base"
    debug = 1

    [profile.copy]
    inherits = "base"
    debug = 1
    "###);
    Ok(())
}

#[test]
fn load_current_settings_rejects_malformed_values() -> anyhow::Result<()> {
    let cases = [
        ("[profile.custom]\ndebug = 1", "", "profile.custom.inherits"),
        ("[profile.dev]\ndebug = [0]", "", "profile.dev.debug"),
        ("", "[build]\nrustc-wrapper = 1", "build.rustc-wrapper"),
        ("", "[build]\nrustflags = [1]", "build.rustflags"),
        ("", "[build]\nrustflags = 1", "build.rustflags"),
    ];
    for (profile, config, error) in cases {
        let mut project = init_cargo_project()?.disable_check_on_drop();
        let manifest = format!("{}\n{profile}\n", project.read_manifest());
        project.manifest(&manifest).config(config);
        let workspace = parse_workspace(&project.manifest_path())?;
        let result = workspace.load_template(&Profile::Custom("custom".to_string()));
        let result = result.and(workspace.load_template(&Profile::dev()));
        assert!(result.unwrap_err().to_string().contains(error), "{error}");
    }
    Ok(())
}

/// Loads the current settings of `profile`, edits them and applies them with replace semantics.
fn edit_current(
    project: &CargoProject,
    profile: &Profile,
    edit: impl FnOnce(&mut Template),
) -> anyhow::Result<ModifiedWorkspace> {
    let workspace = parse_workspace(&project.manifest_path())?;
    let mut template = workspace.load_template(profile)?;
    edit(&mut template);
    workspace.apply(profile, &template, true)
}

fn apply(project: &CargoProject, profile: &str, template: &str) -> anyhow::Result<()> {
    project
        .cmd(&["apply", template, profile, "--nightly=off"])
        .run()?
        .assert_ok();
    Ok(())
}
