<h1 align="center">
    <strong>type-lib</strong>
    <br>
    <sup><sub>VALIDATED DOMAIN TYPES FOR RUST</sub></sup>
</h1>

<p align="center">
    <a href="https://crates.io/crates/type-lib"><img alt="crates.io" src="https://img.shields.io/crates/v/type-lib.svg"></a>
    <a href="https://docs.rs/type-lib"><img alt="docs.rs" src="https://docs.rs/type-lib/badge.svg"></a>
    <a href="https://github.com/jamesgober/type-lib/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/type-lib/actions/workflows/ci.yml/badge.svg"></a>
    <a href="#license"><img alt="license" src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg"></a>
</p>

<p align="center">Parse-dont-validate as a first-class citizen. Domain types with invariants enforced at construction. Zero-overhead wrappers.</p>

---

## Status

**Active development.** Scaffolded and on the path to 1.0. See [.dev/ROADMAP.md](.dev/ROADMAP.md) for milestone tracking.

The public API is not yet stable. Pin specific versions; expect changes pre-1.0.

---

## What it does

Validation and type constraint library. Declare domain types with invariants enforced at construction. Parse-dont-validate pattern as a first-class citizen. Zero-overhead wrappers with derive macros.

---

## Quick start

```toml
[dependencies]
type-lib = "0.1"
```

---

## Standards

- **REPS** governs every decision. See [REPS.md](REPS.md).
- **MSRV:** Rust 1.75.
- **Edition:** 2024.
- **Cross-platform:** Linux, macOS, Windows.

---

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>. All rights reserved.</sub>