<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <strong>type-lib</strong>
    <br>
    <sup><sub>VALIDATED DOMAIN TYPES FOR RUST</sub></sup>
</h1>

<p align="center">
    <a href="https://crates.io/crates/type-lib"><img alt="crates.io" src="https://img.shields.io/crates/v/type-lib.svg"></a>
    <a href="https://crates.io/crates/type-lib"><img alt="downloads" src="https://img.shields.io/crates/d/type-lib.svg"></a>
    <a href="https://docs.rs/type-lib"><img alt="docs.rs" src="https://docs.rs/type-lib/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md" title="MSRV"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.75%2B-blue"></a>
    <a href="https://github.com/jamesgober/type-lib/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/type-lib/actions/workflows/ci.yml/badge.svg"></a>
</p>

<p align="center">Parse-dont-validate as a first-class citizen. Domain types with invariants enforced at construction. Zero-overhead wrappers.</p>


## What it does

`type-lib` is currently the project scaffold for a validated-domain-type crate.
The long-term design is a parse-dont-validate toolkit for invariant-bearing
newtypes, but `v0.1.0` intentionally keeps the shipped API minimal while the
foundation is being finalized.

Today, the crate exposes a single public constant:

- `type_lib::VERSION` - the compile-time package version reported by Cargo.

What is already in place:

- cross-platform CI for Linux, macOS, and Windows
- REPS-aligned lint gates for shipping code
- `std` by default with `no_std` compatibility when the default feature is disabled
- documentation and release structure for subsequent milestones

What is not in `v0.1.0` yet:

- validated wrapper types
- construction and parsing APIs
- public error types and traits
- derive macros

This keeps the first release honest: the crate is publishable scaffolding, not a
feature-complete type system library.

---

## Current API

### `VERSION`

The crate exports its package version as a `&'static str`:

```rust
assert_eq!(type_lib::VERSION, env!("CARGO_PKG_VERSION"));
```

Typical uses include startup banners, diagnostics, and smoke tests:

```rust
let banner = format!("type-lib {}", type_lib::VERSION);
assert!(banner.contains(type_lib::VERSION));
```

---

## Quick start

```toml
[dependencies]
type-lib = "0.1.0"
```

```rust
fn main() {
  println!("using type-lib {}", type_lib::VERSION);
}
```

For the exhaustive API reference, see [docs/API.md](docs/API.md).

---

## Standards

- **REPS** governs every decision. See [REPS.md](REPS.md).
- **MSRV:** Rust 1.75.
- **Edition:** 2021.
- **Cross-platform:** Linux, macOS, Windows.

---

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.



<!-- FOOT COPYRIGHT
################################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>JAMES GOBER.</strong></sup>
</div>