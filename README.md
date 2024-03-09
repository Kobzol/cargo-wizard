# cargo-wizard [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://github.com/kobzol/cargo-wizard/actions/workflows/check.yml/badge.svg

[actions]: https://github.com/kobzol/cargo-wizard/actions?query=branch%3Amain

[Latest Version]: https://img.shields.io/crates/v/cargo-wizard.svg

[crates.io]: https://crates.io/crates/cargo-wizard

Cargo subcommand that applies [profile](https://doc.rust-lang.org/cargo/reference/profiles.html)
and [config](https://doc.rust-lang.org/cargo/reference/config.html#configuration-format) templates to your Cargo project
to configure it for maximum performance, fast compile times or minimal binary size.

![Demo of cargo-wizard](img/wizard-demo.gif)

# Motivation
I often see Rust users asking online about how can they best configure Cargo get e.g. the fastest compilation times,
best
runtime performance or minimal binary size. While this information can be found in
various [books](https://nnethercote.github.io/perf-book/build-configuration.html), [repositories](https://github.com/johnthagen/min-sized-rust)
or [blog posts](https://kobzol.github.io/rust/rustc/2023/10/21/make-rust-compiler-5percent-faster.html), it is annoying
to hunt for it everytime we want to configure a new Cargo project.

This tool tries to automate that process to make it easier.

# Installation

```bash
$ cargo install cargo-wizard
```

# Usage

- Interactive mode (CLI dialog that guides you through the process):
    ```bash
    $ cargo wizard
    ```
- Non-interactive mode (directly apply a predefined template to your Cargo workspace):
    ```bash
    $ cargo wizard apply <template> <profile>
    # For example, apply `fast-runtime` template to the `dist` profile
    $ cargo wizard apply fast-runtime dist
    ```

> You can enable profile/config options that require a nightly compiler by running `cargo-wizard` with a nightly Cargo
> (e.g. `cargo +nightly wizard`) or by using the `--nightly` flag.

# Features
`cargo-wizard` can create or modify Cargo profiles in your `Cargo.toml` manifest and RUSTFLAGS in
the [`.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html#configuration-format) file) based on a
set of prepared templates:

- **`fast-compile`** - minimizes compilation times
    - Disables debuginfo generation and uses a faster linker.
    - In nightly mode, it also enables
      the [Cranelift codegen backend](https://nnethercote.github.io/perf-book/build-configuration.html#cranelift-codegen-back-end)
      and
      the [parallel frontend](https://nnethercote.github.io/perf-book/build-configuration.html#experimental-parallel-front-end).
- **`fast-runtime`** - maximizes runtime performance
    - Enables [LTO](https://doc.rust-lang.org/cargo/reference/profiles.html#lto) and maximal optimization settings.
- **`min-size`** - minimizes binary size
    - Similar to `fast-runtime`, but uses optimization flags designed for small binary size.

You can also modify these templates in the interactive mode to build your own custom template.

## Caveats

- The configuration applied by this tool is quite opinionated and might not fit all use-cases
  perfectly. `cargo-wizard` mostly serves to improve *discoverability* of possible Cargo profile and config options, to
  help you find the ideal settings for your use-cases.
- `cargo-wizard` currently only modifies `Cargo.toml` and `config.toml`. There are other things that can be configured
  to achieve e.g. even smaller binaries, but these are out of scope for this tool, at least at the moment.
- `cargo-wizard` currently ignores Cargo settings that are not relevant to performance.
- Cargo config (`config.toml`) changes are applied to the global `build.hostflags` setting, because per-profile
  RUSTFLAGS are still [unstable](https://github.com/rust-lang/cargo/issues/10271).

# Inspiration

- [Min-sized Rust](https://github.com/johnthagen/min-sized-rust)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/build-configuration.html)

> Why `wizard`? The name is inspired by
> GUI [wizards](https://documentation.softwareag.com/natural/nat913win/edis/edis_win_dia_wiz.htm) that guide you through
> some process using a series of dialogs.

# Contributing
Contributions are welcome :)

Possible future features:

- [ ] Allow configuring
  the [memory allocator](https://nnethercote.github.io/perf-book/build-configuration.html#alternative-allocators).
- [ ] Load/store templates on disk to make them easier to share

# Acknowledgements

- [`toml_edit`](https://docs.rs/toml_edit/latest/toml_edit/): awesome crate that can modify TOML files while keeping
  their original formatting.
- [`inquire`](https://github.com/mikaelmello/inquire): pretty slick crate for building interactive TUI dialogs and
  prompts.

# License
[MIT](LICENSE)
