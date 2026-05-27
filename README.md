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

`type-lib` makes invalid states unrepresentable. Instead of validating a value
every time it is used, you validate it **once** — at construction — and carry a
type that the compiler will only let exist in a valid state. Functions that take
such a type are freed from defensive checks: the type system already did them.

The foundation is two pieces that compose:

- [`Validator`](docs/API.md#validator) — a reusable, type-level validation rule.
  It lives on a zero-sized marker type and is selected through the type system,
  so it carries no state and no storage.
- [`Refined`](docs/API.md#refined) — a `#[repr(transparent)]` wrapper holding a
  value proven to satisfy a `Validator`. It has the exact size and layout of the
  value it wraps, so the guarantee is free at runtime.

A ready-made [`ValidationError`](docs/API.md#validationerror) covers rules that
need only a code and a message; rules that need structured failures define their
own error type through `Validator::Error`.

---

## Features

- **Parse, don't validate** — invariants are enforced at construction and proven
  by the type thereafter; no re-checking at call sites.
- **Zero-overhead wrappers** — `Refined<T, V>` is `#[repr(transparent)]` over `T`
  and stores nothing extra. The validated type is the same size as the raw one.
- **Reusable, type-level rules** — write a `Validator` once and apply it to any
  value type through the type system.
- **Tamper-proof by construction** — `Refined` exposes no `&mut` to its inner
  value and no public field, so a validated value cannot be mutated into an
  invalid one behind the type's back.
- **Bring your own error** — use the bundled `ValidationError` or any custom
  error type via the `Validator::Error` associated type.
- **`no_std` friendly** — the core API is identical with or without `std`; the
  only `std`-gated item is the [`std::error::Error`] impl on `ValidationError`.
- **Cross-platform** — Linux, macOS, and Windows on stable and MSRV 1.75.

---

## API Overview

For the complete reference with examples, see [docs/API.md](docs/API.md).

- [`Validator`](docs/API.md#validator) — reusable, type-level validation rule
- [`Refined`](docs/API.md#refined) — zero-cost wrapper around a validated value
- [`ValidationError`](docs/API.md#validationerror) — ready-made `no_std` error
- [`prelude`](docs/API.md#prelude) — convenient re-exports
- [`VERSION`](docs/API.md#version) — compile-time crate version

---

## Installation

```toml
[dependencies]
type-lib = "0.2.0"

# no_std build
type-lib = { version = "0.2.0", default-features = false }
```

MSRV: Rust 1.75.

## Quick start

```rust
use type_lib::{Refined, ValidationError, Validator};

// A rule, written once and reused anywhere through the type system.
struct NonEmpty;

impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().is_empty() {
            Err(ValidationError::new("non_empty", "value must not be empty"))
        } else {
            Ok(())
        }
    }
}

// A domain type that structurally cannot be empty.
type Username = Refined<String, NonEmpty>;

fn main() -> Result<(), ValidationError> {
    let user = Username::new("alice".to_owned())?;
    assert_eq!(user.len(), 5); // deref to the inner String

    assert!(Username::new(String::new()).is_err());
    Ok(())
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