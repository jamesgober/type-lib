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
//! ## Example
//!
//! ```rust
//! use type_lib::{Refined, ValidationError, Validator};
//!
//! // A rule, written once and reused anywhere through the type system.
//! struct NonEmpty;
//!
//! impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
//!     type Error = ValidationError;
//!
//!     fn validate(value: &S) -> Result<(), Self::Error> {
//!         if value.as_ref().is_empty() {
//!             Err(ValidationError::new("non_empty", "value must not be empty"))
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! // A domain type that structurally cannot be empty.
//! type Username = Refined<String, NonEmpty>;
//!
//! let user = Username::new("alice".to_owned());
//! assert!(user.is_ok());
//! assert!(Username::new(String::new()).is_err());
//! ```
//!
//! ## Cargo features
//!
//! - `std` *(default)* — implements [`std::error::Error`] for [`ValidationError`].
//!   Disable it (`default-features = false`) to build for `no_std`; the core
//!   [`Validator`] / [`Refined`] API is identical either way.
//!
//! ## Stability
//!
//! `v0.2.0` establishes the public API that 1.0 will preserve. Built-in rule sets
//! and a derive macro are planned for later milestones and will be additive.
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

mod error;
mod refined;
mod validator;

pub mod prelude;

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
