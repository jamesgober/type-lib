# Changelog

All notable changes to `type-lib` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- No unreleased changes.

### Changed

- No unreleased changes.

### Fixed

- No unreleased changes.

### Security

- No unreleased changes.

---

## [0.1.0] - 2026-05-27

### Added

- Initial scaffold and repository bootstrap.
- `type_lib::VERSION`, the compile-time crate version, as the sole public item.
- Canonical REPS standards in `REPS.md`.
- Crate-root lint gate denying `unwrap`/`expect`/`todo`/`unimplemented`, the
  `print`/`dbg` family, `unreachable`, and undocumented unsafe.
- CI for Linux/macOS/Windows on stable and MSRV (1.75) running fmt, clippy,
  tests, and rustdoc with `-D warnings`.
- Crate-level documentation describing the scaffolded public surface.
- API reference for the complete `v0.1.0` public API.
- Release note for the scaffold milestone under `docs/release`.

### Changed

- README claims were reduced to the features actually shipped in `v0.1.0`.
- Public documentation now treats the crate as scaffold-only instead of implying future APIs already exist.
- `REPS.md` now holds the canonical Rust Efficiency & Performance Standards in
  place of the bootstrap placeholder.
- CI helper actions pinned to releases running on the Node 24 runtime
  (`actions/cache@v5`) to remove the Node 20 deprecation warnings.

### Fixed

- Set the crate edition to `2021`. The scaffold declared `edition = "2024"`,
  which requires Rust 1.85 and is incompatible with the stated MSRV of 1.75;
  the manifest failed to parse on every toolchain until this was reconciled.
- Normalized source files to UTF-8 without a byte-order mark and with a
  trailing newline so `cargo fmt --all -- --check` passes.
- Added a `.gitattributes` enforcing LF line endings on checkout. The Windows
  CI runner (`git autocrlf=true`) was checking source out with CRLF, which
  failed `cargo fmt --all -- --check` because `rustfmt.toml` pins
  `newline_style = "Unix"`. Linux and macOS were unaffected.
- Aligned the `rustfmt.toml` `edition` with the crate edition (`2021`).

[Unreleased]: https://github.com/jamesgober/type-lib/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jamesgober/type-lib/releases/tag/v0.1.0