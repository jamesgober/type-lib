<h1 align="center">
		<img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
		<br>
		<b>type-lib</b>
		<br>
		<sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
		<sup>
				<a href="../README.md" title="Project Home"><b>HOME</b></a>
				<span>&nbsp;│&nbsp;</span>
				<span>API</span>
		</sup>
</div>

<br>

`type-lib` is at the scaffold milestone in `v0.1.0`. The future validated-type
surface described by the roadmap has not shipped yet, so this document records
the complete public API that exists today rather than the API planned for later
releases.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Public API](#public-api)
	- [`VERSION`](#version)
- [Feature Flags](#feature-flags)
- [Semantics and Compatibility](#semantics-and-compatibility)

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
type-lib = "0.1.0"
```

To build without the standard library, disable default features:

```toml
[dependencies]
type-lib = { version = "0.1.0", default-features = false }
```

## Quick Start

Minimal use in an application:

```rust
fn main() {
		println!("type-lib {}", type_lib::VERSION);
}
```

Smoke-test style version assertion:

```rust
#[test]
fn version_matches_cargo_metadata() {
		assert_eq!(type_lib::VERSION, env!("CARGO_PKG_VERSION"));
}
```

## Public API

### `VERSION`

Signature:

```rust
pub const VERSION: &str
```

Description:

- Exposes the package version embedded by Cargo at compile time.
- Useful for diagnostics, generated banners, telemetry tags, and validation in tests.
- Available in both the default `std` configuration and `no_std` builds.

Parameters:

- None. `VERSION` is a constant, not a function.

Returns:

- `&'static str` containing the crate version from `Cargo.toml`.

Examples:

Use the constant in runtime output:

```rust
let banner = format!("type-lib {}", type_lib::VERSION);
assert!(banner.ends_with(type_lib::VERSION));
```

Use the constant as part of a compatibility check:

```rust
fn crate_series() -> &'static str {
		if type_lib::VERSION.starts_with("0.1.") {
				"scaffold"
		} else {
				"future"
		}
}

assert_eq!(crate_series(), "scaffold");
```

Use the constant in a `no_std`-friendly context:

```rust
let version = type_lib::VERSION;
assert!(!version.is_empty());
```

## Feature Flags

### `std` (default)

- Enabled by default.
- When disabled, the crate builds in `no_std` mode.
- `v0.1.0` does not expose any `std`-only public functions; the feature only
	controls the crate environment while the public API is still minimal.

## Semantics and Compatibility

- `v0.1.0` is the scaffolding release. It establishes packaging, lint, CI, and
	documentation conventions.
- The validated-type API promised by the roadmap is not implemented yet and is
	therefore intentionally absent from this reference.
- The public API recorded here is exhaustive for `v0.1.0`.
