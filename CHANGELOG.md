# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/Kobzol/cargo-wizard/compare/v0.2.2...v0.2.3) - 2025-12-10

### Other

- Suggest lld when it makes sense
- *(deps)* Update dependencies
- Modify headings in README
- Add a warning about using `-Ctarget-cpu=native`
- Fix typo in README
# Dev

- Add a warning that binaries compiled using `-Ctarget-cpu=native` might not be
  portable (https://github.com/Kobzol/cargo-wizard/issues/17).

# 0.2.2 (11. 3. 2024)

- Detect if a linker is installed and don't display warning if it is (https://github.com/Kobzol/cargo-wizard/issues/5).
- Validate profile name (https://github.com/Kobzol/cargo-wizard/issues/7).
- Add support for the `incremental` profile attribute.
- Set all performance-related default properties from the base profile (dev/release) on
  templates (https://github.com/Kobzol/cargo-wizard/issues/4).
- Add support for the `split-debuginfo` profile attribute.
- Improve overwriting of RUSTFLAGS in `.cargo/config.toml` (https://github.com/Kobzol/cargo-wizard/issues/6).

# 0.2.1 (10. 3. 2024)

- Add Unix-specific options.
- Fix cancellation of prompts.
- Unify colors in dialog.

# 0.2.0 (9. 3. 2024)

- Add interactive dialog.
- Add support for modifying `.cargo/config.toml`.
- Add many new template items.

# 0.1.0 (3. 3. 2024)

- Initial release.
