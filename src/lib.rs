//! # type-lib
//!
//! Parse-dont-validate domain types for Rust, with zero-overhead wrappers.
//!
//! `type-lib` turns runtime invariants into compile-time guarantees. Rather than
//! re-checking a value everywhere it is used, you check it **once**, at
//! construction, and carry a type that can only exist in a valid state. Functions
//! that accept such a type are freed from defensive validation: the type system
//! has already done it.
//!
//! ## The foundation
//!
//! Two pieces compose to express "a value that is known to be valid":
//!
//! - [`Validator`] — a reusable, type-level validation rule. It is implemented on
//!   a zero-sized marker type and selected through the type system, so it carries
//!   no state and adds no storage.
//! - [`Refined`] — a `#[repr(transparent)]` wrapper holding a value proven to
//!   satisfy a [`Validator`]. It has the same size and layout as the value it
//!   wraps, so the guarantee is free at runtime.
//!
//! A ready-made [`ValidationError`] covers rules that need only a code and a
//! message; rules that need structured failures define their own error type via
//! [`Validator::Error`].
//!
//! ## Built-in rules and combinators
//!
//! You rarely need to hand-write a rule. The [`rules`] module ships the common
//! ones — length ([`NonEmpty`](rules::NonEmpty), [`MaxLen`](rules::MaxLen),
//! [`LenRange`](rules::LenRange), …), numeric ([`Positive`](rules::Positive),
//! [`InRange`](rules::InRange), …), and string content
//! ([`Ascii`](rules::Ascii), [`Alphanumeric`](rules::Alphanumeric),
//! [`Trimmed`](rules::Trimmed)). The [`combinator`] module composes them at the
//! type level with [`And`](combinator::And), [`Or`](combinator::Or), and
//! [`Not`](combinator::Not).
//!
//! ## Example
//!
//! ```rust
//! use type_lib::combinator::And;
//! use type_lib::rules::{LenRange, Trimmed};
//! use type_lib::Refined;
//!
//! // A username: 3–16 characters with no surrounding whitespace.
//! type Username<'a> = Refined<&'a str, And<Trimmed, LenRange<3, 16>>>;
//!
//! let user = Username::new("alice");
//! assert!(user.is_ok());
//! assert!(Username::new("  x  ").is_err()); // whitespace + too short
//! ```
//!
//! Writing a bespoke rule is just as direct when the built-ins do not fit:
//!
//! ```rust
//! use type_lib::{Refined, ValidationError, Validator};
//!
//! struct Even;
//!
//! impl Validator<i64> for Even {
//!     type Error = ValidationError;
//!
//!     fn validate(value: &i64) -> Result<(), Self::Error> {
//!         if value % 2 == 0 {
//!             Ok(())
//!         } else {
//!             Err(ValidationError::new("even", "value must be even"))
//!         }
//!     }
//! }
//!
//! type EvenI64 = Refined<i64, Even>;
//! assert!(EvenI64::new(4).is_ok());
//! assert!(EvenI64::new(5).is_err());
//! ```
//!
//! ## Cargo features
//!
//! - `std` *(default)* — implies `alloc` and implements [`std::error::Error`] for
//!   [`ValidationError`].
//! - `alloc` — enables the length rules for owned `String` / `Vec<T>` values.
//!
//! With no features (`default-features = false`), the crate is `no_std` and the
//! core [`Validator`] / [`Refined`] API plus all borrowed-value rules are
//! available unchanged.
//!
//! ## Stability
//!
//! The public API established in `v0.2.0` is the surface 1.0 will preserve;
//! `v0.5.0` adds the rule and combinator sets additively. A derive macro for
//! generating validated newtypes is planned for a later milestone.
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

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod refined;
mod validator;

pub mod combinator;
pub mod prelude;
pub mod rules;

pub use crate::error::ValidationError;
pub use crate::refined::Refined;
pub use crate::validator::Validator;

/// Crate version string, populated by Cargo at build time.
///
/// Useful for diagnostics, startup banners, and tests that need to assert the
/// crate metadata seen by Cargo.
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
