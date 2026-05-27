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
- **Built-in rules** — ready-made validators for length
  ([`NonEmpty`](docs/API.md#length-rules), `MinLen`, `MaxLen`, `LenRange`), numbers
  (`Positive`, `InRange`, …), and string content (`Ascii`, `Alphanumeric`,
  `Trimmed`).
- **Composition** — combine rules at the type level with
  [`And`](docs/API.md#combinators), `Or`, and `Not`; the result is itself a rule.
- **Derive macro** — `#[derive(Validated)]` (the `derive` feature) turns a newtype
  into a named domain type with a checked constructor.
- **Reusable, type-level rules** — write a `Validator` once and apply it to any
  value type through the type system.
- **Tamper-proof by construction** — `Refined` exposes no `&mut` to its inner
  value and no public field, so a validated value cannot be mutated into an
  invalid one behind the type's back.
- **Bring your own error** — use the bundled `ValidationError` or any custom
  error type via the `Validator::Error` associated type.
- **`no_std` friendly** — the core API and all borrowed-value rules work without
  `std`; `alloc` adds owned-type (`String` / `Vec`) length rules and `std` adds
  the [`std::error::Error`] impl.
- **Cross-platform** — Linux, macOS, and Windows on stable and MSRV 1.75.

---

## Performance

Validation is the only runtime cost; the wrapper adds none. Local Criterion means
(Windows x86_64, Rust stable, `cargo bench`):

- `Refined::new` with `Ascii` on a short `&str`: **~0.9 ns**
- `Refined::new` with `LenRange<3, 16>` on a `&str`: **~2.2 ns**
- `Refined::new` with `InRange<0, 100>` on an `i32`: **~2.6 ns**
- `get` / `Deref` accessors: **sub-nanosecond** (compile down to a field read)

---

## API Overview

For the complete reference with examples, see [docs/API.md](docs/API.md).

- [`Validator`](docs/API.md#validator) — reusable, type-level validation rule
- [`Refined`](docs/API.md#refined) — zero-cost wrapper around a validated value
- [`ValidationError`](docs/API.md#validationerror) — ready-made `no_std` error
- [Built-in rules](docs/API.md#built-in-rules) — length, numeric, and string rules
- [Combinators](docs/API.md#combinators) — `And`, `Or`, `Not`
- [`Validated`](docs/API.md#validated-derive) — `#[derive]` for validated newtypes (`derive` feature)
- [`prelude`](docs/API.md#prelude) — convenient re-exports
- [`VERSION`](docs/API.md#version) — compile-time crate version

Runnable demos live in [`examples/`](examples): `quick_start`, `built_in_rules`,
`composing_rules`, `custom_rule`, and `derive_newtype`
(e.g. `cargo run --example quick_start`).

---

## Installation

```toml
[dependencies]
type-lib = "0.9.0"

# with the derive macro
type-lib = { version = "0.9.0", features = ["derive"] }

# no_std build (core API + borrowed-value rules)
type-lib = { version = "0.9.0", default-features = false }

# no_std + owned-type rules (String / Vec)
type-lib = { version = "0.9.0", default-features = false, features = ["alloc"] }
```

MSRV: Rust 1.75.

## Quick start

```rust
use type_lib::combinator::And;
use type_lib::rules::{LenRange, Trimmed};
use type_lib::Refined;

// A username: 3–16 characters with no surrounding whitespace.
// Built once, the type guarantees the invariant everywhere it is used.
type Username = Refined<String, And<Trimmed, LenRange<3, 16>>>;

fn main() {
    let user = Username::new("alice".to_owned());
    assert!(user.is_ok());

    assert!(Username::new("ab".to_owned()).is_err());        // too short
    assert!(Username::new("  alice  ".to_owned()).is_err()); // whitespace
}
```

Prefer a distinct named type with its own constructor? Enable the `derive`
feature and annotate a newtype:

```rust
use type_lib::combinator::And;
use type_lib::rules::{LenRange, Trimmed};
use type_lib::Validated;

#[derive(Validated)]
#[valid(And<Trimmed, LenRange<3, 16>>)]
pub struct Username(String);

let user = Username::new("alice".to_owned());
assert!(user.is_ok());
```

Need a rule the built-ins don't cover? Implement [`Validator`](docs/API.md#validator)
on a marker type — see the [`custom_rule`](examples/custom_rule.rs) example.

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