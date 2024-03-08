use anyhow::Context;
use cargo_wizard::{get_core_count, TemplateItemId, TomlValue};
use std::env;
use std::ffi::OsString;
use std::process::{Command, Stdio};

#[derive(Copy, Clone)]
pub enum TomlValueKind {
    Int,
    String,
}

impl TomlValueKind {
    fn matches_value(&self, value: &TomlValue) -> bool {
        match self {
            TomlValueKind::Int if matches!(value, TomlValue::Int(_)) => true,
            TomlValueKind::String if matches!(value, TomlValue::String(_)) => true,
            TomlValueKind::Int | TomlValueKind::String => false,
        }
    }
}

pub enum SelectedPossibleValue {
    Constant { index: usize, value: TomlValue },
    Custom { value: TomlValue },
    None,
}

pub struct TemplateItemMedata {
    values: Vec<PossibleValue>,
    custom_value: Option<CustomPossibleValue>,
    requires_nightly: bool,
}

impl TemplateItemMedata {
    pub fn get_selected_value(&self, value: TomlValue) -> SelectedPossibleValue {
        if let Some(index) = self.values.iter().position(|v| v.value == value) {
            return SelectedPossibleValue::Constant { value, index };
        } else if let Some(custom) = &self.custom_value {
            if custom.kind().matches_value(&value) {
                return SelectedPossibleValue::Custom { value };
            }
        }
        SelectedPossibleValue::None
    }

    pub fn get_possible_values(&self) -> &[PossibleValue] {
        &self.values
    }

    pub fn get_custom_value(&self) -> Option<&CustomPossibleValue> {
        self.custom_value.as_ref()
    }

    pub fn requires_nightly(&self) -> bool {
        self.requires_nightly
    }
}

pub struct CustomPossibleValue {
    kind: TomlValueKind,
    possible_entries: Vec<String>,
}

impl CustomPossibleValue {
    pub fn kind(&self) -> TomlValueKind {
        self.kind
    }

    pub fn possible_entries(&self) -> &[String] {
        &self.possible_entries
    }
}

impl From<TomlValueKind> for CustomPossibleValue {
    fn from(kind: TomlValueKind) -> Self {
        Self {
            kind,
            possible_entries: vec![],
        }
    }
}

#[derive(Default)]
struct MetadataBuilder {
    values: Vec<PossibleValue>,
    custom_value: Option<CustomPossibleValue>,
    requires_nightly: bool,
}

impl MetadataBuilder {
    fn build(self) -> TemplateItemMedata {
        let MetadataBuilder {
            values,
            custom_value,
            requires_nightly,
        } = self;
        TemplateItemMedata {
            values,
            custom_value,
            requires_nightly,
        }
    }

    fn value(mut self, description: &str, value: TomlValue) -> Self {
        self.values.push(PossibleValue::new(description, value));
        self
    }

    fn int(self, description: &str, value: i64) -> Self {
        self.value(description, TomlValue::Int(value))
    }

    fn bool(self, description: &str, value: bool) -> Self {
        self.value(description, TomlValue::Bool(value))
    }

    fn string(self, description: &str, value: &str) -> Self {
        self.value(description, TomlValue::String(value.to_string()))
    }

    fn custom_value<V: Into<CustomPossibleValue>>(mut self, value: V) -> Self {
        self.custom_value = Some(value.into());
        self
    }

    fn requires_nightly(mut self) -> Self {
        self.requires_nightly = true;
        self
    }
}

fn get_target_cpu_list() -> anyhow::Result<Vec<String>> {
    let cmd = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(cmd)
        .args(["--print", "target-cpus"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Cannot spawn `rustc` to find `target-cpus` list")?
        .wait_with_output()
        .context("Cannot run `rustc` to find `target-cpus` list")?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("Cannot run `rustc` to find `target-cpus` list (exit code {})\nStdout:\n{stdout}\n\nStderr:\n{stderr}", output.status));
    }
    Ok(parse_target_cpu_list(&stdout))
}

fn parse_target_cpu_list(input: &str) -> Vec<String> {
    input
        .lines()
        .skip(1)
        .filter_map(|l| l.trim().split_ascii_whitespace().next())
        .map(|l| l.to_string())
        .collect()
}

/// Known options from Cargo, containing descriptions and possible values.
pub struct KnownCargoOptions {
    core_count: i64,
    cpu_list: Vec<String>,
}

impl KnownCargoOptions {
    pub fn create() -> anyhow::Result<Self> {
        let core_count = get_core_count();
        let cpu_list = get_target_cpu_list()?;
        Ok(Self {
            core_count,
            cpu_list,
        })
    }

    pub fn get_all_ids() -> Vec<TemplateItemId> {
        vec![
            TemplateItemId::OptimizationLevel,
            TemplateItemId::Lto,
            TemplateItemId::CodegenUnits,
            TemplateItemId::TargetCpuInstructionSet,
            TemplateItemId::Panic,
            TemplateItemId::DebugInfo,
            TemplateItemId::Strip,
            TemplateItemId::Linker,
            TemplateItemId::CodegenBackend,
            TemplateItemId::FrontendThreads,
        ]
    }

    pub fn get_metadata(&self, id: TemplateItemId) -> TemplateItemMedata {
        match id {
            TemplateItemId::OptimizationLevel => MetadataBuilder::default()
                .int("No optimizations", 0)
                .int("Basic optimizations", 1)
                .int("Some optimizations", 2)
                .int("All optimizations", 3)
                .string("Optimize for small size", "s")
                .string("Optimize for even smaller size", "z")
                .build(),
            TemplateItemId::Lto => MetadataBuilder::default()
                .string("Disable LTO", "off")
                .bool("Thin local LTO", false)
                .string("Thin LTO", "thin")
                .bool("Fat LTO", true)
                .build(),
            TemplateItemId::CodegenUnits => MetadataBuilder::default()
                .int("1 CGU", 1)
                .custom_value(TomlValueKind::Int)
                .build(),
            TemplateItemId::Panic => MetadataBuilder::default()
                .string("Unwind", "unwind")
                .string("Abort", "abort")
                .build(),
            TemplateItemId::DebugInfo => MetadataBuilder::default()
                .bool("Disable debuginfo", false)
                .string("Enable line directives", "line-directives-only")
                .string("Enable line tables", "line-tables-only")
                .int("Limited debuginfo", 1)
                .bool("Full debuginfo", true)
                .build(),
            TemplateItemId::Strip => MetadataBuilder::default()
                .bool("Do not strip anything", false)
                .string("Strip debug info", "debuginfo")
                .string("Strip symbols", "symbols")
                .bool("Strip debug info and symbols", true)
                .build(),
            TemplateItemId::TargetCpuInstructionSet => MetadataBuilder::default()
                .string("Native (best for the local CPU)", "native")
                .custom_value(CustomPossibleValue {
                    kind: TomlValueKind::String,
                    possible_entries: self.cpu_list.clone(),
                })
                .build(),
            TemplateItemId::CodegenBackend => MetadataBuilder::default()
                .string("LLVM", "llvm")
                .string("Cranelift", "cranelift")
                .requires_nightly()
                .build(),
            TemplateItemId::FrontendThreads => MetadataBuilder::default()
                .int(
                    &format!("{} (local core count)", self.core_count),
                    self.core_count,
                )
                .requires_nightly()
                .custom_value(TomlValueKind::Int)
                .build(),
            TemplateItemId::Linker => MetadataBuilder::default()
                .string("LLD", "lld")
                .string("MOLD", "mold")
                .build(),
        }
    }
}

/// Possible value of a Cargo profile or a Cargo config, along with a description of what it does.
#[derive(Debug, Clone)]
pub struct PossibleValue {
    description: String,
    value: TomlValue,
}

impl PossibleValue {
    fn new(description: &str, value: TomlValue) -> Self {
        Self {
            value,
            description: description.to_string(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn value(&self) -> &TomlValue {
        &self.value
    }
}

/// Test that the predefined templates can be created without panicking.
#[cfg(test)]
mod tests {
    use crate::dialog::known_options::{parse_target_cpu_list, KnownCargoOptions};

    #[test]
    fn get_profile_id_possible_values() {
        let options = KnownCargoOptions::create().unwrap();
        for id in KnownCargoOptions::get_all_ids() {
            assert!(!options.get_metadata(id).get_possible_values().is_empty());
        }
    }

    #[test]
    fn test_parse_target_cpu_list() {
        let cpu_list = parse_target_cpu_list(
            r#"Available CPUs for this target:
    native                  - Select the CPU of the current host (currently icelake-client).
    alderlake
    amdfam10
    athlon
    athlon-4
    athlon-fx
    athlon-mp
    athlon-tbird
    athlon-xp
    athlon64
    athlon64-sse3
    atom
    atom_sse4_2
    atom_sse4_2_movbe
    barcelona
    bdver1
    bdver2
    bdver3
    bdver4
    bonnell
    broadwell
    btver1
    btver2
    c3
    c3-2
    cannonlake
    cascadelake
    cooperlake
    core-avx-i
    core-avx2
    core2
    core_2_duo_sse4_1
    core_2_duo_ssse3
    core_2nd_gen_avx
    core_3rd_gen_avx
    core_4th_gen_avx
    core_4th_gen_avx_tsx
    core_5th_gen_avx
    core_5th_gen_avx_tsx
    core_aes_pclmulqdq
    core_i7_sse4_2
    corei7
    corei7-avx
    emeraldrapids
    generic
    geode
    goldmont
    goldmont-plus
    goldmont_plus
    grandridge
    graniterapids
    graniterapids-d
    graniterapids_d
    haswell
    i386
    i486
    i586
    i686
    icelake-client
    icelake-server
    icelake_client
    icelake_server
    ivybridge
    k6
    k6-2
    k6-3
    k8
    k8-sse3
    knl
    knm
    lakemont
    meteorlake
    mic_avx512
    nehalem
    nocona
    opteron
    opteron-sse3
    penryn
    pentium
    pentium-m
    pentium-mmx
    pentium2
    pentium3
    pentium3m
    pentium4
    pentium4m
    pentium_4
    pentium_4_sse3
    pentium_ii
    pentium_iii
    pentium_iii_no_xmm_regs
    pentium_m
    pentium_mmx
    pentium_pro
    pentiumpro
    prescott
    raptorlake
    rocketlake
    sandybridge
    sapphirerapids
    sierraforest
    silvermont
    skx
    skylake
    skylake-avx512
    skylake_avx512
    slm
    tigerlake
    tremont
    westmere
    winchip-c6
    winchip2
    x86-64                  - This is the default target CPU for the current build target (currently x86_64-unknown-linux-gnu).
    x86-64-v2
    x86-64-v3
    x86-64-v4
    yonah
    znver1
    znver2
    znver3
    znver4
"#,
        );
        insta::assert_debug_snapshot!(cpu_list, @r###"
        [
            "native",
            "alderlake",
            "amdfam10",
            "athlon",
            "athlon-4",
            "athlon-fx",
            "athlon-mp",
            "athlon-tbird",
            "athlon-xp",
            "athlon64",
            "athlon64-sse3",
            "atom",
            "atom_sse4_2",
            "atom_sse4_2_movbe",
            "barcelona",
            "bdver1",
            "bdver2",
            "bdver3",
            "bdver4",
            "bonnell",
            "broadwell",
            "btver1",
            "btver2",
            "c3",
            "c3-2",
            "cannonlake",
            "cascadelake",
            "cooperlake",
            "core-avx-i",
            "core-avx2",
            "core2",
            "core_2_duo_sse4_1",
            "core_2_duo_ssse3",
            "core_2nd_gen_avx",
            "core_3rd_gen_avx",
            "core_4th_gen_avx",
            "core_4th_gen_avx_tsx",
            "core_5th_gen_avx",
            "core_5th_gen_avx_tsx",
            "core_aes_pclmulqdq",
            "core_i7_sse4_2",
            "corei7",
            "corei7-avx",
            "emeraldrapids",
            "generic",
            "geode",
            "goldmont",
            "goldmont-plus",
            "goldmont_plus",
            "grandridge",
            "graniterapids",
            "graniterapids-d",
            "graniterapids_d",
            "haswell",
            "i386",
            "i486",
            "i586",
            "i686",
            "icelake-client",
            "icelake-server",
            "icelake_client",
            "icelake_server",
            "ivybridge",
            "k6",
            "k6-2",
            "k6-3",
            "k8",
            "k8-sse3",
            "knl",
            "knm",
            "lakemont",
            "meteorlake",
            "mic_avx512",
            "nehalem",
            "nocona",
            "opteron",
            "opteron-sse3",
            "penryn",
            "pentium",
            "pentium-m",
            "pentium-mmx",
            "pentium2",
            "pentium3",
            "pentium3m",
            "pentium4",
            "pentium4m",
            "pentium_4",
            "pentium_4_sse3",
            "pentium_ii",
            "pentium_iii",
            "pentium_iii_no_xmm_regs",
            "pentium_m",
            "pentium_mmx",
            "pentium_pro",
            "pentiumpro",
            "prescott",
            "raptorlake",
            "rocketlake",
            "sandybridge",
            "sapphirerapids",
            "sierraforest",
            "silvermont",
            "skx",
            "skylake",
            "skylake-avx512",
            "skylake_avx512",
            "slm",
            "tigerlake",
            "tremont",
            "westmere",
            "winchip-c6",
            "winchip2",
            "x86-64",
            "x86-64-v2",
            "x86-64-v3",
            "x86-64-v4",
            "yonah",
            "znver1",
            "znver2",
            "znver3",
            "znver4",
        ]
        "###);
    }
}
