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
//! ## Deriving validated newtypes
//!
//! With the `derive` feature, `#[derive(Validated)]` generates a named domain
//! type from a one-field tuple struct, enforcing a [`Validator`] at construction:
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! use type_lib::rules::InRange;
//! use type_lib::Validated;
//!
//! #[derive(Validated)]
//! #[valid(InRange<0, 100>)]
//! pub struct Percent(i32);
//!
//! assert!(Percent::new(50).is_ok());
//! assert!(Percent::new(150).is_err());
//! # }
//! ```
//!
//! ## Cargo features
//!
//! - `std` *(default)* — implies `alloc` and implements `std::error::Error` for
//!   [`ValidationError`].
//! - `alloc` — enables the length rules for owned `String` / `Vec<T>` values.
//! - `derive` — enables the `Validated` derive macro (see [Deriving validated
//!   newtypes](#deriving-validated-newtypes)).
//!
//! With no features (`default-features = false`), the crate is `no_std` and the
//! core [`Validator`] / [`Refined`] API plus all borrowed-value rules are
//! available unchanged.
//!
//! ## Stability
//!
//! `v1.0.0` is the stable API. The public surface is frozen under SemVer: no
//! breaking change ships without a `2.0`. New rules, combinators, and trait impls
//! arrive as additive minor releases. The error codes returned by the built-in
//! rules are stable across `1.x`.
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

/// Derives a validated newtype: a one-field tuple struct gains a checked
/// constructor, accessors, and `Deref`, enforcing a [`Validator`] at construction.
///
/// Available with the `derive` feature. Annotate the struct with
/// `#[valid(<Validator>)]`, where the validator is any rule implementing
/// [`Validator`] for the field type — a [built-in rule](crate::rules), a
/// [combinator], or your own.
///
/// The generated `new` returns `Result<Self, <V as Validator<T>>::Error>`; `get`,
/// `into_inner`, `Deref`, and `AsRef` expose the inner value. The field stays
/// private, so construction must go through `new`.
///
/// # Examples
///
/// ```rust
/// use type_lib::combinator::And;
/// use type_lib::rules::{LenRange, Trimmed};
/// use type_lib::Validated;
///
/// #[derive(Validated)]
/// #[valid(And<Trimmed, LenRange<3, 16>>)]
/// pub struct Username(String);
///
/// let user = Username::new("alice".to_owned()).expect("valid");
/// assert_eq!(user.get(), "alice");
/// assert!(Username::new("  ".to_owned()).is_err());
/// ```
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use type_lib_derive::Validated;

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
