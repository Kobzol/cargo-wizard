# cargo-wizard [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://github.com/kobzol/cargo-wizard/actions/workflows/check.yml/badge.svg

[actions]: https://github.com/kobzol/cargo-wizard/actions?query=branch%3Amain

[Latest Version]: https://img.shields.io/crates/v/cargo-wizard.svg

[crates.io]: https://crates.io/crates/cargo-wizard

**Cargo subcommand that applies predefined [Cargo profile](https://doc.rust-lang.org/cargo/reference/profiles.html)
templates to get you up to speed quickly.**

# Motivation
I often see Rust users asking online how can they best configure Cargo get e.g. the fastest compilation times, best
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

```bash
# Create or modify a Cargo profile based on a template
$ cargo wizard apply dist fast-runtime
```

# Features
`cargo-wizard` can create or modify Cargo profiles in your `Cargo.toml` manifest (and also other
configuration in the [`.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html#configuration-format)
file) based on a set of prepared templates:

- **`fast-compile`** - profile template that tries to minimize compilation times
- **`fast-runtime`** - profile template that tries to maximize runtime performance
- **`min-size`** - profile template that tries to minimize binary size

It uses the incredible [`toml_edit`](https://docs.rs/toml_edit/latest/toml_edit/) crate to keep original formatting of
the modified TOML files.

# Inspiration

- [Min-sized Rust](https://github.com/johnthagen/min-sized-rust)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/build-configuration.html)

# Contributions
Contributions are welcome :)

# License
[MIT](LICENSE)
