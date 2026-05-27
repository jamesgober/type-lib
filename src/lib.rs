//! # type-lib
//!
//! Validated domain types for Rust.
//!
//! `type-lib` is in the scaffold phase. The crate currently exposes only build and
//! package metadata while the public API for validated domain types is finalized.
//! The long-term goal remains parse-dont-validate domain modeling with zero-cost
//! wrapper types, but those facilities are not part of `v0.1.0` yet.
//!
//! ## Current API surface
//!
//! The current public API consists of:
//!
//! - [`VERSION`], the crate version embedded at compile time.
//!
//! ## Example
//!
//! ```rust
//! assert_eq!(type_lib::VERSION, env!("CARGO_PKG_VERSION"));
//! ```
//!
//! ## Status
//!
//! `v0.1.0` establishes the repository scaffold, lint policy, CI workflow, and
//! documentation structure. The validated type constructors, error types, traits,
//! and derive support planned for later milestones are intentionally absent.
//!
//! # License
//!
//! Dual-licensed under Apache-2.0 OR MIT.

#![doc(html_root_url = "https://docs.rs/type-lib")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![deny(unused_results)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::unreachable)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]

/// Crate version string, populated by Cargo at build time.
///
/// This is the only public item exposed by the `v0.1.0` scaffold. It is useful
/// for diagnostics, startup banners, and tests that need to assert the crate
/// metadata seen by Cargo.
///
/// # Examples
///
/// Compare the exported value with Cargo's package version:
///
/// ```rust
/// assert_eq!(type_lib::VERSION, env!("CARGO_PKG_VERSION"));
/// ```
///
/// Embed the version in generated output:
///
/// ```rust
/// let banner = format!("type-lib {}", type_lib::VERSION);
/// assert!(banner.starts_with("type-lib "));
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
