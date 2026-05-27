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

## [0.5.0] - 2026-05-27

The implementation milestone: built-in rules, composition, property tests, and
benchmarks. Purely additive over `v0.2.0`.

### Added

- `rules` module of ready-made validators, all reporting `ValidationError`:
  - length (via the new `HasLength` trait): `NonEmpty`, `MinLen<MIN>`,
    `MaxLen<MAX>`, `LenRange<MIN, MAX>` — counting `char`s for strings and
    elements for slices/vectors.
  - numeric: `Positive`, `NonNegative`, `Negative`, `NonPositive` (signed integer
    and float primitives), and `InRange<MIN, MAX>` (`i64`-bounded, for the integer
    types that fit losslessly in `i64`).
  - string: `Ascii`, `Alphanumeric`, `Trimmed` (any `AsRef<str>`).
- `combinator` module: `And<A, B>`, `Or<A, B>`, and `Not<A>`, composing rules at
  the type level. `And`/`Or` share their sub-rules' error type; `Not` reports
  `ValidationError`.
- `alloc` Cargo feature, enabling the length rules for owned `String` / `Vec<T>`
  values. `std` now implies `alloc`.
- `And`, `Or`, and `Not` re-exported from the `prelude`.
- `proptest` property tests covering rule/combinator invariants
  (`tests/proptests.rs`).
- `criterion` benchmarks for the validation hot path (`benches/validation.rs`).
- Runnable examples under `examples/`: `quick_start`, `built_in_rules`,
  `composing_rules`, `custom_rule`.

### Changed

- README and `docs/API.md` expanded to document the rule and combinator sets with
  examples, and to record the new `alloc` feature.
- Committed `Cargo.lock` pins the dev-dependency tree (proptest 1.4, criterion's
  clap 4.5, tempfile 3.14, half 2.4) to versions that build on the MSRV (1.75).

---

## [0.2.0] - 2026-05-27

The foundation milestone: the public API surface that `1.0` will preserve.

### Added

- `Validator<T>` trait — a reusable, type-level validation rule with an
  associated `Error` type and a single `validate` method.
- `Refined<T, V>` — a `#[repr(transparent)]` wrapper holding a value proven to
  satisfy a `Validator`. Constructed only through the validating `Refined::new`;
  exposes `get`, `into_inner`, `Deref`, and `AsRef`, and delegates `Clone`,
  `Copy`, `Debug`, `Display`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`
  to the inner value (bounded on `T`, never on the marker `V`).
- `ValidationError` — a `Copy`, allocation-free, `no_std` error carrying a stable
  `code` and a human-readable `message`, with a `const` constructor and a
  `Display` of `"<code>: <message>"`. Implements `std::error::Error` under `std`.
- `prelude` module re-exporting `Refined`, `Validator`, and `ValidationError`.
- Unit tests on every public item plus an end-to-end integration suite
  (`tests/foundation.rs`).

### Changed

- Crate-level documentation now describes the parse-dont-validate foundation and
  its usage instead of the scaffold-only surface.
- README and `docs/API.md` rewritten to cover the full `v0.2.0` API with
  examples for each item and common pattern.

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

[Unreleased]: https://github.com/jamesgober/type-lib/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/jamesgober/type-lib/compare/v0.2.0...v0.5.0
[0.2.0]: https://github.com/jamesgober/type-lib/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/type-lib/releases/tag/v0.1.0