# cargo-wizard [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://github.com/kobzol/cargo-wizard/actions/workflows/check.yml/badge.svg

[actions]: https://github.com/kobzol/cargo-wizard/actions?query=branch%3Amain

[Latest Version]: https://img.shields.io/crates/v/cargo-wizard.svg

[crates.io]: https://crates.io/crates/cargo-wizard

**Cargo subcommand that applies predefined [Cargo profile](https://doc.rust-lang.org/cargo/reference/profiles.html)
templates to your Cargo workspace to get you up to speed quickly.**

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

# Features
`cargo-wizard` can create or modify Cargo profiles in your `Cargo.toml` manifest (and also configuration in
the [`.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html#configuration-format) file) based on a
set of prepared templates:

- **`fast-compile`** - minimizes compilation times
- **`fast-runtime`** - maximizes runtime performance
- **`min-size`** - minimizes binary size

In the interactive mode, it also allows you to customize the templates.

> Note that `config.toml` changes are applied to the global `build.hostflags` option, because per-profile Rustflags are
> still [unstable](https://github.com/rust-lang/cargo/issues/10271).

# Inspiration

- [Min-sized Rust](https://github.com/johnthagen/min-sized-rust)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/build-configuration.html)

# Contributions
Contributions are welcome :)

Possible features:

- [ ] Load/store templates on disk to make them easier to share

# Acknowledgements

`cargo wizard` uses the incredible [`toml_edit`](https://docs.rs/toml_edit/latest/toml_edit/) crate to keep original
formatting of
the modified TOML files.

# License
[MIT](LICENSE)
